use serde::Deserialize;

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  // your custom commands
  // multiple arguments are allowed
  // note that rename_all = "camelCase": you need to use "myCustomCommand" on JS
  LoadImages { paths: Vec<String>, callback: String, error: String },
  RunMerge { lightframes: Vec<String>, mode_str: String, out_path: String, callback: String, error: String}
}
