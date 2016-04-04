use io::types::*;

/// A comparison between two values.
///
/// # JSON
///
/// A range is an object with one field `{key: value}`.
///
#[derive(Clone, PartialEq, PartialOrd, Debug)]
pub enum Range {
    /// Leq(x) accepts any value v such that v <= x.
    ///
    /// # JSON
    ///
    /// ```
    /// extern crate foxbox_taxonomy;
    /// extern crate serde_json;
    ///
    /// use foxbox_taxonomy::values::*;
    /// use foxbox_taxonomy::parse::*;
	/// use foxbox_taxonomy::serialize::*;
    ///
    /// # fn main() {
    ///
    /// let source = "{
    ///   \"Leq\": { \"OnOff\": \"On\" }
    /// }";
    ///
    /// let parsed = Range::from_str(source).unwrap();
    /// if let Range::Leq(ref leq) = parsed {
    ///   assert_eq!(*leq, Value::OnOff(OnOff::On));
    /// } else {
    ///   panic!();
    /// }
    ///
    /// let as_json = parsed.to_json(&mut MultiPart::new());
    /// let as_string = serde_json::to_string(&as_json).unwrap();
    /// assert_eq!(as_string, "{\"Leq\":{\"OnOff\":\"On\"}}");
    ///
    /// # }
    /// ```
    Leq(Value),

    /// Geq(x) accepts any value v such that v >= x.
    Geq(Value),

    /// BetweenEq {min, max} accepts any value v such that `min <= v`
    /// and `v <= max`. If `max < min`, it never accepts anything.
    BetweenEq { min: Value, max: Value },

    /// OutOfStrict {min, max} accepts any value v such that `v < min`
    /// or `max < v`
    OutOfStrict { min: Value, max: Value },

    /// Eq(x) accespts any value v such that v == x
    Eq(Value),
}

impl Range {
    /// Determine if a value is accepted by this range.
    pub fn contains(&self, value: &Value) -> bool {
        use self::Range::*;
        match *self {
            Leq(ref max) => value <= max,
            Geq(ref min) => value >= min,
            BetweenEq { ref min, ref max } => min <= value && value <= max,
            OutOfStrict { ref min, ref max } => value < min || max < value,
            Eq(ref val) => value == val,
        }
    }
}
