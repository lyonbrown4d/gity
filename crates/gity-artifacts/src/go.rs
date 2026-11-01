use crate::ArtifactValidationError;

pub fn go_escape_path(value: &str) -> String {
  let mut output = String::with_capacity(value.len());
  for character in value.chars() {
    if character == '!' {
      output.push_str("!!");
    } else if character.is_ascii_uppercase() {
      output.push('!');
      output.push(character.to_ascii_lowercase());
    } else {
      output.push(character);
    }
  }
  output
}

pub fn go_unescape_path(value: &str) -> Result<String, ArtifactValidationError> {
  let mut output = String::with_capacity(value.len());
  let mut characters = value.chars();
  while let Some(character) = characters.next() {
    if character != '!' {
      if character.is_ascii_uppercase() {
        return Err(ArtifactValidationError::new(
          "Go module path is not escaped",
        ));
      }
      output.push(character);
      continue;
    }
    match characters.next() {
      Some('!') => output.push('!'),
      Some(next) if next.is_ascii_lowercase() => output.push(next.to_ascii_uppercase()),
      _ => {
        return Err(ArtifactValidationError::new(
          "Go module path escape is invalid",
        ));
      }
    }
  }
  Ok(output)
}

#[cfg(test)]
#[path = "go_tests.rs"]
mod tests;
