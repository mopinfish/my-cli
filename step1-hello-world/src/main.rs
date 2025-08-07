use clap::{Arg, Command};

fn main() {
    let matches = Command::new("hello-cli")
        .version("0.1.0")
        .about("A simple Hello World CLI tool")
        .author("Otsuka Noboru <mopinfish@gmail.ocm>")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Name to greet")
                .required(false)
        )
        .arg(
            Arg::new("count")
                .short('c')
                .long("count")
                .value_name("NUMBER")
                .help("Number of times to greet")
                .default_value("1")
                .value_parser(clap::value_parser!(u32))
        )
        .arg(
            Arg::new("uppercase")
                .short('u')
                .long("uppercase")
                .help("Display greeting in uppercase")
                .action(clap::ArgAction::SetTrue)
        )
        .get_matches();

    // 引数の取得
    let name = matches.get_one::<String>("name")
        .map(|s| s.as_str())  // String を &str に変換
        .unwrap_or("World");  // デフォルト値は文字列リテラル
    let count = matches.get_one::<u32>("count").unwrap();
    let uppercase = matches.get_flag("uppercase");

    // グリーティングメッセージの作成
    let message = if uppercase {
        format!("HELLO, {}!", name.to_uppercase())
    } else {
        format!("Hello, {}!", name)
    };

    // 指定された回数だけメッセージを表示
    for i in 1..=*count {
        if *count > 1 {
            println!("{} ({})", message, i);
        } else {
            println!("{}", message);
        }
    }
}
