use std::cmp::{min, max};

use super::builders;
use super::convert::AsLayoutNode;
use super::{Alignment, Layout, LayoutNode, LayoutSettings, LayoutVariant, Style, ColorChange};

use font;
use font::constants::*;
use font::glyph_metrics;
use font::kerning::{superscript_kern, subscript_kern};
use font::variant::Variant;
use font::{AtomType, Symbol, VariantGlyph, FontUnit};
use layout;
use layout::convert::Scaled;
use layout::spacing::{atom_space, Spacing};
use parser::nodes::{BarThickness, MathStyle, ParseNode, Accent, Delimited, GenFraction, Radical,
                    Scripts, Stack};
use environments::Array;

/// Entry point to our recursive algorithm
pub fn layout(nodes: &[ParseNode], config: LayoutSettings) -> Layout {
    layout_recurse(nodes, config, AtomType::Transparent)
}

/// This method takes the parsing nodes and layouts them to layout nodes.
#[allow(unconditional_recursion)]
fn layout_recurse(nodes: &[ParseNode],
                  mut config: LayoutSettings,
                  parent_next: AtomType)
                  -> Layout {
    let mut result = Layout::new();
    let mut prev = AtomType::Transparent;

    for idx in 0..nodes.len() {
        let node = &nodes[idx];

        // To determine spacing between glyphs, we look at each pair and their types.
        // Obtain the atom_type from the next node,  if we are the last in the node
        // list then we obtain the atomtype from the next node in parent's list.
        let next = match nodes.get(idx + 1) {
            Some(node) => node.atom_type(),
            None => parent_next,
        };

        let mut current = node.atom_type();
        if current == AtomType::Binary {
            if prev == AtomType::Transparent || prev == AtomType::Binary ||
               prev == AtomType::Relation || prev == AtomType::Open ||
               prev == AtomType::Punctuation {
                current = AtomType::Alpha;
            } else if let AtomType::Operator(_) = prev {
                current = AtomType::Alpha;
            } else if next == AtomType::Relation || next == AtomType::Close ||
                      next == AtomType::Punctuation {
                current = AtomType::Alpha;
            }
        }

        let sp = atom_space(prev, current, config.style);
        if sp != Spacing::None {
            let kern = sp.to_unit().scaled(config);
            result.add_node(kern!(horz: kern));
        }

        prev = current;
        match *node {
            ParseNode::Style(sty) => config.style = sty,
            _ => dispatch(&mut result, config, node, next),
        }
    }

    result.finalize()
}

fn layout_node(node: &ParseNode, config: LayoutSettings) -> Layout {
    let mut result = Layout::new();
    dispatch(&mut result, config, node, AtomType::Transparent);
    result.finalize()
}

fn dispatch(lay: &mut Layout, config: LayoutSettings, node: &ParseNode, next: AtomType) {
    match *node {
        ParseNode::Symbol(sym) => symbol(lay, sym, config),
        ParseNode::Scripts(ref script) => scripts(lay, script, config),
        ParseNode::Radical(ref rad) => radical(lay, rad, config),
        ParseNode::Delimited(ref delim) => delimited(lay, delim, config),
        ParseNode::Accent(ref acc) => accent(lay, acc, config),
        ParseNode::GenFraction(ref f) => frac(lay, f, config),
        ParseNode::Stack(ref stack) => substack(lay, stack, config),
        ParseNode::Array(ref arr) => array(lay, arr, config),

        ParseNode::AtomChange(ref ac) => lay.add_node(layout(&ac.inner, config).as_node()),
        ParseNode::Group(ref gp) => lay.add_node(layout(gp, config).as_node()),
        ParseNode::Rule(rule) => lay.add_node(rule.as_layout(config)),
        ParseNode::Kerning(kern) => lay.add_node(kern!(horz: kern.scaled(config))),

        ParseNode::Color(ref clr) => {
            let inner = layout_recurse(&clr.inner, config, next);
            lay.add_node(color!(inner, clr))
        }

        _ => warn!("ignored ParseNode: {:?}", node),
    }
}

fn symbol(result: &mut Layout, sym: Symbol, config: LayoutSettings) {
    // Operators are handled specially.  We may need to find a larger
    // symbol and vertical center it.
    if let AtomType::Operator(_) = sym.atom_type {
        largeop(result, sym, config);
    } else {
        let glyph = font::glyph_metrics(sym.unicode);
        result.add_node(glyph.as_layout(config));
    }
}

fn largeop(result: &mut Layout, sym: Symbol, config: LayoutSettings) {
    let glyph = font::glyph_metrics(sym.unicode);
    if config.style > Style::Text {
        let axis_offset = AXIS_HEIGHT.scaled(config);
        let largeop = glyph
            .vert_variant(DISPLAY_OPERATOR_MIN_HEIGHT)
            .as_layout(config);
        let shift = (largeop.height + largeop.depth) / 2 - axis_offset;
        result.add_node(vbox!(offset: shift; largeop));
    } else {
        result.add_node(glyph.as_layout(config));
    }
}

fn accent(result: &mut Layout, acc: &Accent, config: LayoutSettings) {
    // [ ] The width of the resulting box is the width of the base.
    // [ ] Bottom accents: vertical placement is directly below nucleus,
    //       no correction takes place.
    // [ ] WideAccent vs Accent: Don't expand Accent types.
    let base = layout(&acc.nucleus, config.cramped());
    let accent_variant = glyph_metrics(acc.symbol.unicode).horz_variant(base.width);
    let accent = accent_variant.as_layout(config);

    // Attachment points for accent & base are calculated by
    //   (a) Non-symbol: width / 2.0,
    //   (b) Symbol:
    //      1. Attachment point (if there is one)
    //      2. Otherwise: (width + ic) / 2.0
    let base_offset = match layout::is_symbol(&base.contents) {
        Some(sym) => {
            let glyph = glyph_metrics(sym.unicode);
            if glyph.attachment != 0.into() {
                glyph.attachment.scaled(config)
            } else {
                let offset: FontUnit = (glyph.advance + glyph.italics) / 2;
                offset.scaled(config)
            }
        }
        None => base.width / 2,
    };

    let acc_offset = match accent_variant {
        VariantGlyph::Replacement(sym) => {
            let glyph = glyph_metrics(sym.unicode);
            if glyph.attachment != FontUnit::from(0) {
                glyph.attachment.scaled(config)
            } else {
                // For glyphs without attachmens, we must
                // also account for combining glyphs
                let offset: FontUnit = (sym.bbox.2 + sym.bbox.0) / 2;
                offset.scaled(config)
            }
        }

        VariantGlyph::Constructable(_, _) => accent.width / 2,
    };

    // Do not place the accent any further than you would if given
    // an `x` character in the current style.
    let delta = -min(base.height, ACCENT_BASE_HEIGHT.scaled(config));

    // By not placing an offset on this vbox, we are assured that the
    // baseline will match the baseline of `base.as_node()`
    result.add_node(vbox!(hbox!(kern!(horz: base_offset - acc_offset), accent),
                          kern!(vert: delta),
                          base.as_node()));
}

fn delimited(result: &mut Layout, delim: &Delimited, config: LayoutSettings) {
    let inner = layout(&delim.inner, config).as_node();

    // Convert inner group dimensions to font unit
    let height = inner.height;
    let depth = inner.depth;

    // Only extend if we meet a certain size
    // TODO: This quick height check doesn't seem to be strong enough,
    // reference: http://tug.org/pipermail/luatex/2010-July/001745.html
    if max(height, -depth) > DELIMITED_SUB_FORMULA_MIN_HEIGHT / 2 {
        let axis = AXIS_HEIGHT;

        let mut clearance = 2 * max(height - axis, axis - depth);
        clearance = max(DELIMITER_FACTOR * clearance,
                        height - depth - DELIMITER_SHORT_FALL);

        let axis = AXIS_HEIGHT.scaled(config);
        let left = match delim.left.unicode {
            46 => kern!(horz: NULL_DELIMITER_SPACE),
            _ => {
                glyph_metrics(delim.left.unicode)
                    .vert_variant(clearance)
                    .as_layout(config)
                    .centered(axis)
            }
        };

        let right = match delim.right.unicode {
            46 => kern!(horz: NULL_DELIMITER_SPACE),
            _ => {
                glyph_metrics(delim.right.unicode)
                    .vert_variant(clearance)
                    .as_layout(config)
                    .centered(axis)
            }
        };

        result.add_node(left);
        result.add_node(inner);
        result.add_node(right);
    } else {
        let left = match delim.left.unicode {
            46 => kern!(horz: NULL_DELIMITER_SPACE),
            _ => glyph_metrics(delim.left.unicode).as_layout(config),
        };

        let right = match delim.right.unicode {
            46 => kern!(horz: NULL_DELIMITER_SPACE),
            _ => glyph_metrics(delim.right.unicode).as_layout(config),
        };

        result.add_node(left);
        result.add_node(inner);
        result.add_node(right);
    }
}

fn scripts(result: &mut Layout, scripts: &Scripts, config: LayoutSettings) {
    // See: https://tug.org/TUGboat/tb27-1/tb86jackowski.pdf
    //      https://www.tug.org/tugboat/tb30-1/tb94vieth.pdf
    let base = match scripts.base {
        Some(ref base) => layout_node(base, config),
        None => Layout::new(),
    };

    let mut sup = match scripts.superscript {
        Some(ref sup) => layout(sup, config.superscript_variant()),
        None => Layout::new(),
    };

    let mut sub = match scripts.subscript {
        Some(ref sub) => layout(sub, config.subscript_variant()),
        None => Layout::new(),
    };

    // We use a different algoirthm for handling scripts for operators with limits.
    // This is where he handle Operators with limits.
    if let Some(ref b) = scripts.base {
        if AtomType::Operator(true) == b.atom_type() {
            operator_limits(result, base, sup, sub, config);
            return
        }
    }

    // We calculate the vertical positions of the scripts.  The `adjust_up`
    // variable will describe how far we need to adjust the superscript up.
    let mut adjust_up = FontUnit::from(0);
    let mut adjust_down = FontUnit::from(0);
    let mut sup_kern = FontUnit::from(0);
    let mut sub_kern = FontUnit::from(0);

    if scripts.superscript.is_some() {
        // Use default font values for first iteration of vertical height.
        adjust_up = match config.style.is_cramped() {
                true => SUPERSCRIPT_SHIFT_UP_CRAMPED,
                false => SUPERSCRIPT_SHIFT_UP,
            }
            .scaled(config);

        // TODO: These checks should be recursive?
        let mut height = base.height;
        if let Some(ref b) = scripts.base {
            if b.atom_type() != AtomType::Operator(false) {
                // For accents whose base is a simple symbol we do not take
                // the accent into account while positioning the superscript.
                if let ParseNode::Accent(ref acc) = **b {
                    use parser::is_symbol;
                    if let Some(sym) = is_symbol(&acc.nucleus) {
                        height = glyph_metrics(sym.unicode).height().scaled(config);
                    }
                }
                // Apply italics correction is base is a symbol
                else if let Some(base_sym) = base.is_symbol() {
                    // Lookup font kerning of superscript is also a symbol
                    if let Some(sup_sym) = sup.is_symbol() {
                        let bg = glyph_metrics(base_sym.unicode);
                        let sg = glyph_metrics(sup_sym.unicode);
                        let kern = superscript_kern(bg, sg, adjust_up).scaled(config);
                        sup_kern = base_sym.italics + kern;
                    } else {
                        sup_kern = base_sym.italics;
                    }
                }
            }
        }

        let drop_max = SUPERSCRIPT_BASELINE_DROP_MAX.scaled(config);
        adjust_up = max!(adjust_up,
                         height - drop_max,
                         SUPERSCRIPT_BOTTOM_MIN.scaled(config) - sup.depth);
    }

    // We calculate the vertical position of the subscripts.  The `adjust_down`
    // variable will describe how far we need to adjust the subscript down.
    if scripts.subscript.is_some() {
        // Use default font values for first iteration of vertical height.
        adjust_down = max!(SUBSCRIPT_SHIFT_DOWN.scaled(config),
                            sub.height - SUBSCRIPT_TOP_MAX.scaled(config),
                            SUBSCRIPT_BASELINE_DROP_MIN.scaled(config) - base.depth);

        // Provided that the base and subscript are symbols, we apply
        // kerning values found in the kerning font table
        if let Some(ref b) = scripts.base {
            if let Some(base_sym) = base.is_symbol() {
                if AtomType::Operator(false) == b.atom_type() {
                    // This recently changed in LuaTeX.  See `nolimitsmode`.
                    // This needs to be the glyph information _after_ layout for base.
                    sub_kern = -glyph_metrics(base_sym.unicode).italics.scaled(config);
                }
            }

            if let (Some(ssym), Some(bsym)) = (sub.is_symbol(), base.is_symbol()) {
                let bg = glyph_metrics(bsym.unicode);
                let sg = glyph_metrics(ssym.unicode);
                sub_kern += subscript_kern(bg, sg, adjust_down).scaled(config);
            }
        }
    }

    // TODO: lazy gap fix; see BottomMaxWithSubscript
    if scripts.subscript.is_some() && scripts.superscript.is_some() {
        let sup_bot = adjust_up + sup.depth;
        let sub_top = sub.height - adjust_down;
        let gap_min = SUB_SUPERSCRIPT_GAP_MIN.scaled(config);
        if sup_bot - sub_top < gap_min {
            let adjust = (gap_min - sup_bot + sub_top) / 2;
            adjust_up += adjust;
            adjust_down += adjust;
        }
    }

    let mut contents = builders::VBox::new();
    if scripts.superscript.is_some() {
        if sup_kern != FontUnit::from(0) {
            sup.contents.insert(0, kern!(horz: sup_kern));
            sup.width += sup_kern;
        }

        let corrected_adjust = adjust_up - sub.height + adjust_down;
        contents.add_node(sup.as_node());
        contents.add_node(kern!(vert: corrected_adjust));
    }

    contents.set_offset(adjust_down);
    if scripts.subscript.is_some() { 
        if sub_kern != FontUnit::from(0) {
            sub.contents.insert(0, kern!(horz: sub_kern));
            sub.width += sub_kern;
        }
        contents.add_node(sub.as_node());
    }

    result.add_node(base.as_node());
    result.add_node(contents.build());
}

fn operator_limits(result: &mut Layout,
                   base: Layout,
                   sup: Layout,
                   sub: Layout,
                   config: LayoutSettings) {
    // Provided that the operator is a simple symbol, we need to account
    // for the italics correction of the symbol.  This how we "center"
    // the superscript and subscript of the limits.
    let delta = match base.is_symbol() {
        Some(gly) => gly.italics,
        None => FontUnit::from(0),
    };

    // Next we calculate the kerning required to separate the superscript
    // and subscript (respectively) from the base.
    let sup_kern = max(UPPER_LIMIT_BASELINE_RISE_MIN.scaled(config),
                       UPPER_LIMIT_GAP_MIN.scaled(config) - sup.depth);
    let sub_kern = max(LOWER_LIMIT_GAP_MIN.scaled(config),
                       LOWER_LIMIT_BASELINE_DROP_MIN.scaled(config) - sub.height) -
                   base.depth;

    // We need to preserve the baseline of the operator when
    // attaching the scripts.  Since the base should already
    // be aligned, we only need to offset by the addition of
    // subscripts.
    let offset = sub.height + sub_kern;

    // We will construct a vbox containing the superscript/base/subscript.
    // We will all of these nodes, so we widen each to the largest.
    let width = max!(base.width, sub.width + delta / 2, sup.width + delta / 2);

    result.add_node(vbox![
        offset: offset;
        hbox![align: Alignment::Centered(sup.width);
            width: width;
            kern![horz: delta / 2],
            sup.as_node()
        ],

        kern!(vert: sup_kern),
        base.centered(width).as_node(),
        kern!(vert: sub_kern),

        hbox![align: Alignment::Centered(sub.width);
            width: width;
            kern![horz: -delta / 2],
            sub.as_node()
        ]
    ]);
}

fn frac(result: &mut Layout, frac: &GenFraction, config: LayoutSettings) {
    let config = match frac.style {
        MathStyle::NoChange => config,
        MathStyle::Display => config.with_display(),
        MathStyle::Text => config.with_text(),
    };

    let bar = match frac.bar_thickness {
        BarThickness::Default => FRACTION_RULE_THICKNESS.scaled(config),
        BarThickness::None => FontUnit::from(0),
        BarThickness::Unit(u) => u.scaled(config),
    };

    let mut n = layout(&frac.numerator, config.numerator());
    let mut d = layout(&frac.denominator, config.denominator());

    if n.width > d.width {
        d.alignment = Alignment::Centered(d.width);
        d.width = n.width;
    } else {
        n.alignment = Alignment::Centered(n.width);
        n.width = d.width;
    }

    let numer = n.as_node();
    let denom = d.as_node();

    let axis = AXIS_HEIGHT.scaled(config);
    let shift_up: FontUnit;
    let shift_down: FontUnit;
    let gap_num: FontUnit;
    let gap_denom: FontUnit;

    if config.style > Style::Text {
        shift_up = FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP.scaled(config);
        shift_down = FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN.scaled(config);
        gap_num = FRACTION_NUM_DISPLAY_STYLE_GAP_MIN.scaled(config);
        gap_denom = FRACTION_DENOM_DISPLAY_STYLE_GAP_MIN.scaled(config);
    } else {
        shift_up = FRACTION_NUMERATOR_SHIFT_UP.scaled(config);
        shift_down = FRACTION_DENOMINATOR_SHIFT_DOWN.scaled(config);
        gap_num = FRACTION_NUMERATOR_GAP_MIN.scaled(config);
        gap_denom = FRACTION_DENOMINATOR_GAP_MIN.scaled(config);
    }

    let kern_num = max(shift_up - axis - bar / 2, gap_num - numer.depth);
    let kern_den = max(shift_down + axis - denom.height - bar / 2, gap_denom);
    let offset = denom.height + kern_den + bar / 2 - axis;

    let width = numer.width;
    let inner = vbox!(offset: offset;
        numer,
        kern!(vert: kern_num),
        rule!(width: width, height: bar),
        kern!(vert: kern_den),
        denom
    );

    // Enclose fraction with delimiters if provided, otherwise with a NULL_DELIMITER_SPACE.
    let left = match frac.left_delimiter {
        None => kern!(horz: NULL_DELIMITER_SPACE),
        Some(sym) => {
            let clearance = 2 * max(inner.height - AXIS_HEIGHT, AXIS_HEIGHT - inner.depth);
            let clearance = max(clearance, DELIMITED_SUB_FORMULA_MIN_HEIGHT);
            let axis = AXIS_HEIGHT.scaled(config);

            glyph_metrics(sym.unicode)
                .vert_variant(clearance)
                .as_layout(config)
                .centered(axis)
        }
    };

    let right = match frac.right_delimiter {
        None => kern!(horz: NULL_DELIMITER_SPACE),
        Some(sym) => {
            let clearance = 2 * max(inner.height - AXIS_HEIGHT, AXIS_HEIGHT - inner.depth);
            let clearance = max(clearance, DELIMITED_SUB_FORMULA_MIN_HEIGHT);
            let axis = AXIS_HEIGHT.scaled(config);

            glyph_metrics(sym.unicode)
                .vert_variant(clearance)
                .as_layout(config)
                .centered(axis)
        }
    };

    result.add_node(left);
    result.add_node(inner);
    result.add_node(right);
}

fn radical(result: &mut Layout, rad: &Radical, config: LayoutSettings) {
    // reference rule 11 from pg 443 of TeXBook
    let contents = layout(&rad.inner, config.cramped()).as_node();

    // obtain minimum clearange between radicand and radical bar
    // and cache other sizes that will be needed
    let gap = match config.style >= Style::Display {
        true => RADICAL_DISPLAY_STYLE_VERTICAL_GAP.scaled(config),
        false => RADICAL_VERTICAL_GAP.scaled(config),
    };

    let rule_thickness = RADICAL_RULE_THICKNESS.scaled(config);
    let rule_ascender = RADICAL_EXTRA_ASCENDER.scaled(config);

    // determine size of radical glyph
    let inner_height = (contents.height - contents.depth) + gap + rule_thickness;
    let sqrt = glyph_metrics(0x221A).vert_variant(inner_height).as_layout(config);

    // pad between radicand and radical bar
    let delta = (sqrt.height - sqrt.depth - inner_height) / 2 + rule_thickness;
    let gap = max(delta, gap);

    // offset radical symbol
    let offset = rule_thickness + gap + contents.height;
    let offset = sqrt.height - offset;

    // padding above sqrt
    // TODO: This is unclear
    let top_padding = rule_ascender - rule_thickness;

    result.add_node(vbox![offset: offset; sqrt]);
    result.add_node(vbox![kern!(vert: top_padding),
                          rule!(width:  contents.width, height: rule_thickness),
                          kern!(vert: gap),
                          contents]);
}

fn substack(result: &mut Layout, stack: &Stack, config: LayoutSettings) {
    // Don't bother constructing a new node if there is nothing.
    if stack.lines.len() == 0 {
        return;
    }

    // Layout each line in the substack, and track which line is the widest
    let mut lines: Vec<Layout> = Vec::with_capacity(stack.lines.len());
    let mut widest = FontUnit::from(0);
    let mut widest_idx = 0;
    for (n, line) in stack.lines.iter().enumerate() {
        let line = layout(line, config);
        if line.width > widest {
            widest = line.width;
            widest_idx = n;
        }
        lines.push(line);
    }

    // Center lines according to widest variant
    for (n, line) in lines.iter_mut().enumerate() {
        if n == widest_idx {
            continue;
        }
        line.alignment = Alignment::Centered(line.width);
        line.width = widest;
    }

    // The line gap will be taken from STACK_GAP constants
    let gap_min = if config.style > Style::Text {
        STACK_DISPLAY_STYLE_GAP_MIN.scaled(config)
    } else {
        STACK_GAP_MIN.scaled(config)
    };

    // No idea.
    let gap_try = if config.style > Style::Text {
            STACK_TOP_DISPLAY_STYLE_SHIFT_UP - AXIS_HEIGHT + STACK_BOTTOM_SHIFT_DOWN -
            2 * ACCENT_BASE_HEIGHT
        } else {
            STACK_TOP_SHIFT_UP - AXIS_HEIGHT + STACK_BOTTOM_SHIFT_DOWN - 2 * ACCENT_BASE_HEIGHT
        }
        .scaled(config);

    // Join the lines with appropriate spacing inbetween
    let mut vbox = builders::VBox::new();
    let length = lines.len();
    let mut prev: FontUnit;
    for (idx, line) in lines.into_iter().enumerate() {
        prev = line.depth;
        vbox.add_node(line.as_node());

        // Try for an ideal gap, otherwise use the minimum
        if idx < length {
            let gap = max(gap_min, gap_try - prev);
            vbox.add_node(kern![vert: gap]);
        }
    }

    // Vertically center the stack to the axis
    let offset = (vbox.height + vbox.depth) / 2 - AXIS_HEIGHT.scaled(config);
    vbox.set_offset(offset);
    result.add_node(vbox.build());
}

fn array(result: &mut Layout, array: &Array, config: LayoutSettings) {
    // TODO: let jot = UNITS_PER_EM / 4;
    let strut_height = UNITS_PER_EM / 10 * 7; // \strutbox height = 0.7\baseline
    let strut_depth = UNITS_PER_EM / 10 * 3; // \strutbox depth  = 0.3\baseline
    let row_sep = UNITS_PER_EM / 4;
    let column_sep = UNITS_PER_EM / 12 * 5;

    // Don't bother constructing a new node if there is nothing.
    let num_rows = array.rows.len();
    let num_columns = array.rows.iter().map(Vec::len).max().unwrap_or(0);
    if num_columns == 0 {
        return;
    }

    let mut columns = Vec::with_capacity(num_columns);
    for _ in 0..num_columns {
        columns.push(Vec::with_capacity(num_rows));
    }

    // Layout each node in each row, while keeping track of the largest row/col
    let mut col_widths = vec![FontUnit::from(0); num_columns];
    let mut row_heights = Vec::with_capacity(num_rows);
    let mut prev_depth = FontUnit::from(0);
    let mut row_max = strut_height;
    for row in &array.rows {
        let mut max_depth = FontUnit::from(0);
        for col_idx in 0..num_columns {
            // layout row element if it exists
            let square = match row.get(col_idx) {
                Some(r) => {
                    // record the max height/width for current row/col
                    let square = layout(r, config);
                    row_max = max(square.height, row_max);
                    max_depth = max(max_depth, -square.depth);
                    col_widths[col_idx] = max(col_widths[col_idx], square.width);
                    square
                },
                _ => Layout::new(),
            };

            columns[col_idx].push(square);
        }

        // ensure row height >= strut_height
        row_heights.push(row_max + prev_depth);
        row_max = strut_height;
        prev_depth = max(0.into(), max_depth - strut_depth);
    }

    // TODO: reference row layout here: crl
    // the body of the matrix is an hbox of column vectors.
    let mut hbox = builders::HBox::new();

    // If there are no delimiters, insert a null space.  Otherwise we insert
    // the delimiters _after_ we have laidout the body of the matrix.
    if array.left_delimiter.is_none() {
        hbox.add_node(kern![horz: NULL_DELIMITER_SPACE]);
    }

    // layout the body of the matrix
    for (col_idx, col) in columns.into_iter().enumerate() {
        let mut vbox = builders::VBox::new();
        for (row_idx, mut row) in col.into_iter().enumerate() {
            // Center columns as necessary
            if row.width < col_widths[col_idx] {
                row.alignment = Alignment::Centered(row.width);
                row.width = col_widths[col_idx];
            }

            // Add additional strut if required to align rows
            if row.height < row_heights[row_idx] {
                let diff = row_heights[row_idx] - row.height;
                vbox.add_node(kern![vert: diff]);
            }

            // add inter-row spacing.  Since vboxes get their depth from the their
            // last entry, we manually add the depth from the last row if it exceeds
            // the row_seperation.
            // FIXME: This should be actual depth, not additional kerning
            let node = row.as_node();
            if row_idx + 1 == num_rows {
                let depth = max(-node.depth, row_sep);
                vbox.add_node(node);
                vbox.add_node(kern![vert: depth]);
            } else {
                vbox.add_node(node);
                vbox.add_node(kern![vert: row_sep]);
            }
        }

        // add column to matrix body and column seperation spacing except for last one.
        hbox.add_node(vbox.build());
        if col_idx + 1 < num_columns {
            hbox.add_node(kern![horz: column_sep]);
        }
    }

    if array.right_delimiter.is_none() {
        hbox.add_node(kern![horz: NULL_DELIMITER_SPACE]);
    }

    // TODO: Reference array vertical alignment (optional [bt] arguments)
    // Vertically center the array on axis.
    // Note: hbox has no depth, so hbox.height is total height.
    let height = hbox.height;
    let mut vbox = builders::VBox::new();
    let offset = height / 2 - AXIS_HEIGHT.scaled(config);
    vbox.set_offset(offset);
    vbox.add_node(hbox.build());
    let vbox = vbox.build();

    // Now that we know the layout of the matrix body we can place scaled delimiters
    // First check if there are any delimiters to add, if not just return.
    if array.left_delimiter.is_none() && array.right_delimiter.is_none() {
        result.add_node(vbox);
        return;
    }

    // place delimiters in an hbox surrounding the matrix body
    let mut hbox = builders::HBox::new();
    let axis = AXIS_HEIGHT.scaled(config);
    let clearance = max(DELIMITER_FACTOR * height,
                        height - DELIMITER_SHORT_FALL);

    if let Some(left) = array.left_delimiter {
        let left = glyph_metrics(left.unicode)
            .vert_variant(clearance)
            .as_layout(config)
            .centered(axis);
        hbox.add_node(left);
    }

    hbox.add_node(vbox);
    if let Some(right) = array.right_delimiter {
        let right = glyph_metrics(right.unicode)
            .vert_variant(clearance)
            .as_layout(config)
            .centered(axis);
        hbox.add_node(right);
    }
    result.add_node(hbox.build());
}
