
use statrs::function::gamma::gamma;

use math::{
  MathResponse,
  MathResult
};

use math::syntax::{
  parse,
  Expr
};

pub struct MathEngine {
}

struct Rational(i64,i64);

impl MathEngine {

  pub fn evaluate(&self, input: &str) -> MathResult {

    match parse(input) {
      Ok(expr) => {
        let (tex, val) = self.evaluate_node(&expr);

        Ok(MathResponse{ tex: tex, value: val })
      }
      _ => Err(format!("Can't evaluate"))
    }
  }

  fn evaluate_node(&self, expr: &Expr) -> (String,f64) {
    match expr {
      Expr::Num(prefix, repeat_opt) => {
        let mut result = match repeat_opt {
          Some(repeat) => {
            let tex = format!("{}\\overbar{{ {} }}", prefix, repeat);
            let val = self.parse_repeated(prefix, repeat);
            (tex, val)
          },
          None => {
            let tex = format!("{}", prefix);
            let val : f64 = prefix.parse().unwrap();
            (tex, val)
          }
        };

        result
      },
      Expr::Add(left, right) => {
        let (left_tex, left_val) = self.evaluate_node(left);
        let (right_tex, right_val) = self.evaluate_node(right);
        let tex = format!("{}+{}", left_tex, right_tex);
        let val = left_val + right_val;
        (tex, val)
      },
      Expr::Sub(left, right) => {
        let (left_tex, left_val) = self.evaluate_node(left);
        let (right_tex, right_val) = self.evaluate_node(right);
        let tex = format!("{}-{}", left_tex, right_tex);
        let val = left_val - right_val;
        (tex, val)
      },
      Expr::Mul(left, right) => {
        let (left_tex, left_val) = self.evaluate_node(left);
        let (right_tex, right_val) = self.evaluate_node(right);
        let tex = format!("{} \\times {}", left_tex, right_tex);
        let val = left_val * right_val;
        (tex, val)
      },
      Expr::Div(left, right) => {
        let (left_tex, left_val) = self.evaluate_node(left);
        let (right_tex, right_val) = self.evaluate_node(right);
        let tex = format!("\\frac{{ {} }}{{ {} }}", left_tex, right_tex);
        let val = left_val / right_val;
        (tex, val)
      },
      Expr::Exp(left, right) => {
        let (left_tex, left_val) = self.evaluate_node(left);
        let (right_tex, right_val) = self.evaluate_node(right);
        let tex = format!("{} ^{{ {} }}", left_tex, right_tex);
        let val = left_val.powf(right_val);
        (tex, val)
      },
      Expr::Paren(inner) => {
        let (inner_tex, val) = self.evaluate_node(inner);
        let tex = format!("\\left( {{ {} }} \\right)", inner_tex);
        (tex, val)
      },
      Expr::Radical(inner) => {
        let (inner_tex, val) = self.evaluate_node(inner);
        let tex = format!("\\sqrt{{ {} }}", inner_tex);
        (tex, val.sqrt())
      },
      Expr::Factorial(inner) => {
        let (inner_tex, inner_val) = self.evaluate_node(inner);

        let val = if inner_val < 0. {
          0. / 0. // f64::NAN doesn't seem to exist?
        }
        else {
          gamma(inner_val)
        };

        let tex = format!("{{ {} }}!", inner_tex);
        (tex, val)
      },
    }
  }

  fn parse_repeated(&self, prefix: &str, repeat: &str) -> f64 {
    let decimal_idx = prefix.find('.')
        .expect("There shouldn't be a repeat number with no decimal");
    let mut decimal_digits = prefix.len() - decimal_idx - 1;
    let repeat_len = repeat.len();
    let mut result = String::from(prefix);

    while decimal_digits < 17 {
      result.push_str(repeat);
      decimal_digits += repeat_len;
    }

    result.parse().unwrap()
  }
}