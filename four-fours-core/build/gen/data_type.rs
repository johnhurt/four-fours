
lazy_static! {

  pub static ref LONG : DataType = DataType::Primitive(
      PrimitiveDataTypeBuilder::default()
          .name("Long")
          .rust_name("i64")
          .swift_name("Int64")
          .build().unwrap()
      );

  pub static ref MUTABLE_BYTE_POINTER : DataType = DataType::Primitive(
      PrimitiveDataTypeBuilder::default()
          .name("MutableBytePointer")
          .rust_name("*mut u8")
          .swift_name("UnsafeMutablePointer<UInt8>?")
          .build().unwrap()
      );

  pub static ref STRING : DataType = DataType::Stringy;
}

#[derive(Serialize,Clone,Copy)]
pub enum DataType {
  Nil,
  Stringy,
  Primitive(PrimitiveDataType),
  RustGeneric(RustGenericDataType),
  RustStruct(RustStructDataType),
  SwiftGeneric(SwiftGenericDataType),
  SwiftStruct(SwiftStructDataType)
}

#[derive(Serialize,Builder,Default,Clone,Copy)]
#[builder(default)]
#[builder(pattern = "owned")]
pub struct PrimitiveDataType {
  pub name: &'static str,
  pub rust_name: &'static str,
  pub swift_name: &'static str
}

#[derive(Serialize,Builder,Default,Clone,Copy)]
#[builder(default)]
#[builder(public)]
#[builder(pattern = "owned")]
pub struct RustGenericDataType {
  pub symbol: Option<&'static str>,
  pub bound_type: RustStructDataType
}

#[derive(Serialize,Builder,Default,Clone,Copy)]
#[builder(default)]
#[builder(public)]
#[builder(pattern = "owned")]
pub struct RustStructDataType {
  pub name: &'static str,
  pub import: Option<&'static str>
}

#[derive(Serialize,Builder,Default,Clone,Copy)]
#[builder(default)]
#[builder(public)]
#[builder(pattern = "owned")]
pub struct SwiftGenericDataType {
  pub symbol: Option<&'static str>,
  pub bound_type: SwiftStructDataType
}

#[derive(Serialize,Builder,Default,Clone,Copy)]
#[builder(default)]
#[builder(public)]
#[builder(pattern = "owned")]
pub struct SwiftStructDataType {
  pub name: &'static str,
  pub import: Option<&'static str>
}

impl DataType {
  pub fn get_imports(&self) -> Vec<String> {
    match &self {
      DataType::Nil => Vec::new(),
      DataType::Stringy => vec![
          String::from("util::RustString")
      ],
      DataType::Primitive(_) => Vec::new(),
      DataType::RustGeneric(generic_type) => {
        generic_type.bound_type.import
            .clone()
            .into_iter()
            .map(String::from)
            .collect()
      },
      DataType::SwiftGeneric(generic_type) => {
        generic_type.bound_type.import
            .clone()
            .into_iter()
            .map(String::from)
            .collect()
      },
      DataType::SwiftStruct(_) => Vec::new(),
      DataType::RustStruct(struct_type) => {
        struct_type.import.iter()
            .map(|import| { import.to_string() })
            .collect()
      }
    }
  }

  pub fn rust_generic(symbol: Option<&'static str>,
      bound_type: DataType) -> DataType {
    if let DataType::RustStruct(rust_type) = bound_type {
      DataType::RustGeneric(RustGenericDataTypeBuilder::default()
          .symbol(symbol)
          .bound_type(rust_type)
          .build().unwrap())
    }
    else { panic!("Can only create a rust generic out of a rust struct") }
  }

  pub fn rust_struct(name: &'static str,
      import: Option<&'static str>) -> DataType {
    DataType::RustStruct(RustStructDataTypeBuilder::default()
        .name(name)
        .import(import)
        .build().unwrap())
  }

  pub fn swift_generic(symbol: Option<&'static str>,
      bound_type: DataType) -> DataType {
    if let DataType::SwiftStruct(swift_type) = bound_type {
      DataType::SwiftGeneric(SwiftGenericDataTypeBuilder::default()
          .symbol(symbol)
          .bound_type(swift_type)
          .build().unwrap())
    }
    else { panic!("Can only create a swift generic out of a swift struct") }
  }

  pub fn swift_struct(name: &'static str,
      import: Option<&'static str>) -> DataType {
    DataType::SwiftStruct(SwiftStructDataTypeBuilder::default()
        .name(name)
        .import(import)
        .build().unwrap())
  }
}

impl Default for DataType {
  fn default() -> DataType { DataType::Nil }
}