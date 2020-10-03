use {
    clap::{value_t, App, Arg},
    futures::stream::StreamExt,
    reqwest,
    tokio::{
        self,
        io::{self, AsyncReadExt},
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Eval args
    let matches = App::new("Fhc")
        .version("0.1.0")
        .author("Eduard Tolosa <edu4rdshl@protonmail.com>")
        .about("Fast HTTP Checker.")
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .takes_value(true)
                .help("Number of threads. Default: 100"),
        )
        .arg(
            Arg::with_name("timeout")
                .long("timeout")
                .takes_value(true)
                .help("Timeout in seconds. Default: 2"),
        )
        .arg(
            Arg::with_name("user-agent")
                .short("u")
                .long("user-agent")
                .takes_value(true)
                .help("User agent string."),
        )
        .arg(
            Arg::with_name("1xx")
                .short("1")
                .long("1xx")
                .takes_value(false)
                .help("Show URLs with 100-199 response codes only."),
        )
        .arg(
            Arg::with_name("2xx")
                .short("2")
                .long("2xx")
                .takes_value(false)
                .help("Show URLs with 200-299 response codes only."),
        )
        .arg(
            Arg::with_name("3xx")
                .short("3")
                .long("3xx")
                .takes_value(false)
                .help("Show URLs with 300-399 response codes only."),
        )
        .arg(
            Arg::with_name("4xx")
                .short("4")
                .long("4xx")
                .takes_value(false)
                .help("Show URLs with 400-499 response codes only."),
        )
        .arg(
            Arg::with_name("5xx")
                .short("5")
                .long("5xx")
                .takes_value(false)
                .help("Show URLs with 500-599 response codes only."),
        )
        .get_matches();

    // Assign values or use defaults
    let conditional_response_code = if matches.is_present("1xx") {
        100
    } else if matches.is_present("2xx") {
        200
    } else if matches.is_present("3xx") {
        300
    } else if matches.is_present("4xx") {
        400
    } else if matches.is_present("5xx") {
        500
    } else {
        0
    };
    let threads = value_t!(matches.value_of("threads"), usize).unwrap_or_else(|_| 100);
    let timeout = value_t!(matches.value_of("timeout"), u64).unwrap_or_else(|_| 2);
    let user_agent = &value_t!(matches.value_of("user-agent"), String).unwrap_or_else(|_| "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/85.0.4183.121 Safari/537.36".to_string());

    // Read stdin
    let mut buffer = String::new();
    let mut stdin = io::stdin();
    stdin.read_to_string(&mut buffer).await?;
    let hosts: Vec<String> = buffer.lines().map(str::to_owned).collect();

    futures::stream::iter(hosts.into_iter().map(|host| async move {
        let mut response_code = 0;
        let mut protocol = String::new();
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .user_agent(user_agent)
            .build()
            .unwrap();
        if let Ok(resp) = client.get(&format!("https://{}", host)).send().await {
            protocol = "https://".to_string();
            response_code = resp.status().as_u16();
        } else if let Ok(resp) = client.get(&format!("http://{}", host)).send().await {
            protocol = "http://".to_string();
            response_code = resp.status().as_u16()
        }
        if !protocol.is_empty() && conditional_response_code == 0 {
            println!("{}", protocol + &host)
        } else if (!protocol.is_empty() && conditional_response_code != 0)
            && (response_code >= conditional_response_code
                && response_code <= conditional_response_code + 99)
        {
            println!("{}", protocol + &host)
        }
    }))
    .buffer_unordered(threads)
    .collect::<Vec<()>>()
    .await;
    Ok(())
}
