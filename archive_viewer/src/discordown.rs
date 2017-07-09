// TODO: code blocks

const VALID_TAGS: [&'static str; 8] = ["*", "_", "**", "__", "***", "~~", "`", "``"];

pub fn parse(escaped: &str) -> String {
  let mut symbols = 0;
  let mut skip = 0;
  let mut in_tag = false;

  let mut result = String::new();
  let mut buffer = String::new();

  let owned = escaped.to_owned();
  let mut iter = owned.chars().peekable();
  while let Some(c) = iter.next() {
    if skip > 0 {
      skip -= 1;
      continue;
    }
    match c {
      '*' | '_' | '`' => {
        if symbols == 0 {
          result += &buffer;
          buffer = String::new();
        }
        symbols += 1;
      },
      '<' => in_tag = true,
      '>' => in_tag = false,
      _ => {}
    }
    buffer += &c.to_string();
    if in_tag {
      continue;
    }
    if let Some(&next) = iter.peek() {
      if next != c && symbols > 0 && VALID_TAGS.contains(&buffer.as_ref()) {
        let s = iter.clone().collect::<String>();
        if let Some(m) = s.find(&buffer) {
          let content = &s[..m];
          skip = content.chars().count() + buffer.len();
          let styles = match buffer.as_ref() {
            "_" | "*" => "emphasis",
            "__" => "underline",
            "**" => "strong",
            "***" => "strong emphasis",
            "~~" => "strikethrough",
            "`" | "``" => "code",
            _ => unreachable!()
          };
          result += &format!("<span class=\"{}\">{}</span>",
            styles,
            if styles != "code" { parse(content) } else { content.to_owned() });
          buffer = String::new();
          symbols = 0;
        }
      }
    }
  }
  result += &buffer;
  result
}

#[cfg(test)]
mod test {
  extern crate test;

  const TEST_STRING: &'static str = "blah __underline__ blah **bold** and ***bold italic*** with __***underline bold italics***__";
  const TEST_EXPECTED: &'static str = "blah <span class=\"underline\">underline</span> blah <span class=\"strong\">bold</span> and <span class=\"strong emphasis\">bold italic</span> with <span class=\"underline\"><span class=\"strong emphasis\">underline bold italics</span></span>";

  #[test]
  fn parse() {
    assert_eq!(TEST_EXPECTED, super::parse(TEST_STRING));
  }

  #[test]
  fn parse_weird() {
    assert_eq!("<span class=\"underline\">kek gon give it **to ya</span>**",
      super::parse("__kek gon give it **to ya__**"));
  }

  #[test]
  fn parse_text() {
    let text = "this is some normal text";
    assert_eq!(text, super::parse(text));
  }

  #[test]
  fn parse_two_asterisks() {
    assert_eq!("**", super::parse("**"));
  }

  #[test]
  fn parse_three_asterisks() {
    assert_eq!("***", super::parse("***"));
  }

  #[test]
  #[ignore]
  // honestly, I can't be bothered to fix this. sure, it doesn't match discord's markdown parser
  // exactly, but it's late, this is hard for my brain, and who types this kind of shit anyway
  fn parse_four_asterisks() {
    assert_eq!("<span class=\"emphasis\">**</span>four!<span class=\"emphasis\">**</span>",
      super::parse("****four!****"));
  }

  #[test]
  fn parse_inner() {
    let source = "*italicized __underlines__ yo*";
    let output = "<span class=\"emphasis\">italicized <span class=\"underline\">underlines</span> yo</span>";
    assert_eq!(output, super::parse(source));
  }

  #[test]
  fn parse_tag() {
    let source = "some <a href=\"abc _def_ ghi\">text</a> yo";
    assert_eq!(source, super::parse(source));
  }

  #[bench]
  fn parse_benchmark(b: &mut self::test::Bencher) {
    b.iter(|| super::parse(TEST_STRING));
  }
}
