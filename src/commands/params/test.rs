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
