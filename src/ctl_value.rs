#[cfg(target_os = "freebsd")]
use temperature::Temperature;

/// An Enum that holds all values returned by sysctl calls.
/// Extract inner value with `if let` or `match`.
///
/// # Example
///
/// ```ignore
/// let val_enum = sysctl::value("kern.osrevision");
///
/// if let sysctl::CtlValue::Int(val) = val_enum {
///     println!("Value: {}", val);
/// }
/// ```
#[derive(Debug, PartialEq, PartialOrd)]
pub enum CtlValue {
    None,
    Node(Vec<u8>),
    Int(i32),
    String(String),
    S64(u64),
    Struct(Vec<u8>),
    Uint(u32),
    Long(i64),
    Ulong(u64),
    U64(u64),
    U8(u8),
    U16(u16),
    S8(i8),
    S16(i16),
    S32(i32),
    U32(u32),
    #[cfg(target_os = "freebsd")]
    Temperature(Temperature),
}

impl std::fmt::Display for CtlValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let s = match self {
            CtlValue::None => "[None]".to_owned(),
            CtlValue::Int(i) => format!("{}", i),
            CtlValue::Uint(i) => format!("{}", i),
            CtlValue::Long(i) => format!("{}", i),
            CtlValue::Ulong(i) => format!("{}", i),
            CtlValue::U8(i) => format!("{}", i),
            CtlValue::U16(i) => format!("{}", i),
            CtlValue::U32(i) => format!("{}", i),
            CtlValue::U64(i) => format!("{}", i),
            CtlValue::S8(i) => format!("{}", i),
            CtlValue::S16(i) => format!("{}", i),
            CtlValue::S32(i) => format!("{}", i),
            CtlValue::S64(i) => format!("{}", i),
            CtlValue::Struct(_) => "[Opaque Struct]".to_owned(),
            CtlValue::Node(_) => "[Node]".to_owned(),
            CtlValue::String(s) => s.to_owned(),
            #[cfg(target_os = "freebsd")]
            CtlValue::Temperature(t) => format!("{}", t.kelvin()),
        };
        write!(f, "{}", s)
    }
}
