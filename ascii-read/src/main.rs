use std::{fs::File, io::{stdout, Read, Write}};

use termion::raw::IntoRawMode;

fn main() {
  let path = "../kirby.out";

  let mut file = File::open(path).expect("No file found");

  let mut content = String::new();
  file.read_to_string(&mut content).unwrap();

  let mut handle = stdout().into_raw_mode().unwrap().lock();

  for char in content.chars() {
    if char == '\n' {
      write!(handle, "\r\n").unwrap();
    } else {
      write!(handle, "{}", char).unwrap();
    }
  }
}
