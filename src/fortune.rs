use std::process::Command;

pub fn get_fortune() -> String {
  let output = Command::new("fortune").arg("-a").output();
  match output {
    Ok(result) if result.status.success() => String::from_utf8_lossy(&result.stdout).into_owned(),
    _ => String::from("Sorry folks, no fortune at the moment!\n--- Fortune writer"),
  }
}

pub fn get_figlet(str: &String) -> String {
  let output = Command::new("figlet").arg("-kf").arg("slant").arg(&str).output();
  match output {
    Ok(result) if result.status.success() => String::from_utf8_lossy(&result.stdout).into_owned(),
    _ => str.to_owned(),
  }
}
