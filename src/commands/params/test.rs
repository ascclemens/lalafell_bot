#[derive(Deserialize, Debug, PartialEq)]
struct Params {
  who: String,
  number: Option<u8>
}

#[test]
fn parse_params() {
  let params: Params = super::from_str("someone 5").unwrap();
  let expected = Params { who: "someone".to_owned(), number: Some(5) };
  assert_eq!(expected, params);
}

#[test]
fn parse_optional_params() {
  let params: Params = super::from_str("someone").unwrap();
  let expected = Params { who: "someone".to_owned(), number: None };
  assert_eq!(expected, params);
}

#[test]
fn parse_missing_params() {
  match super::from_str::<Params>("") {
    Err(super::error::Error::MissingParams) => {},
    _ => panic!()
  }
}

#[test]
#[should_panic]
fn parse_invalid_params() {
  super::from_str::<Params>("someone bad").unwrap();
}

#[test]
fn parse_vec() {
  #[derive(Debug, Deserialize, PartialEq)]
  struct VecParams {
    first: String,
    others: Vec<String>
  }
  let params: VecParams = super::from_str("first second third").unwrap();
  let expected = VecParams { first: "first".to_owned(), others: vec!["second".to_owned(), "third".to_owned()] };
  assert_eq!(expected, params);
}
