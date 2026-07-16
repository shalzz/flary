use clap::{Arg, ArgAction, Command};
use clap_complete::{generate, Shell};

use flary::commands;
use flary::settings::global_user::get_valid_user;

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::client::async_api::Client;
use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::Environment;

fn build_app() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about("Manage Cloudflare domains and DNS records")
        .arg_required_else_help(true)
        .subcommand(
            Command::new("config")
                .about("Manage authentication configuration")
                .subcommand(
                    Command::new("auth")
                        .about("Authenticate with Cloudflare via OAuth (DNS read/write scopes)"),
                ),
        )
        .subcommand(
            Command::new("domains")
                .alias("domain")
                .about("Manage Cloudflare domains/zones")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("ls")
                        .alias("list")
                        .about("List all domains"),
                ),
        )
        .subcommand(
            Command::new("dns")
                .about("Manage DNS records for a domain")
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("ls")
                        .alias("list")
                        .about("List DNS records for a domain")
                        .arg(
                            Arg::new("name")
                                .help("Domain name")
                                .required(true),
                        ),
                )
                .subcommand(
                    Command::new("add")
                        .about("Add a DNS record")
                        .long_about("Add a DNS record to a domain.\nSupported record types: A, AAAA, CNAME, TXT, MX, SRV, NS")
                        .args(&[
                            Arg::new("domain")
                                .help("Domain to add the record to")
                                .required(true),
                            Arg::new("name")
                                .help("Record name (e.g. www, @, mail)")
                                .required(true),
                            Arg::new("type")
                                .help("Record type (A, AAAA, CNAME, TXT, MX, SRV, NS)")
                                .required(true),
                            Arg::new("value")
                                .help("Record value (IP, hostname, or text)")
                                .required(true),
                            Arg::new("proxied")
                                .long("proxied")
                                .help("Proxy through Cloudflare (orange cloud)")
                                .action(ArgAction::SetTrue),
                            Arg::new("ttl")
                                .long("ttl")
                                .help("TTL in seconds (1 = automatic)")
                                .default_value("1"),
                            Arg::new("priority")
                                .long("priority")
                                .help("Priority for MX/SRV records"),
                        ]),
                )
                .subcommand(
                    Command::new("update")
                        .about("Update a DNS record")
                        .long_about("Update an existing DNS record by ID.\nUse `dns ls` to find the record ID.")
                        .args(&[
                            Arg::new("id")
                                .help("DNS record ID to update")
                                .required(true),
                            Arg::new("domain")
                                .help("Domain the record belongs to")
                                .required(true),
                            Arg::new("name")
                                .help("Record name (e.g. www, @, mail)")
                                .required(true),
                            Arg::new("type")
                                .help("Record type (A, AAAA, CNAME, TXT, MX, SRV, NS)")
                                .required(true),
                            Arg::new("value")
                                .help("Record value (IP, hostname, or text)")
                                .required(true),
                            Arg::new("proxied")
                                .long("proxied")
                                .help("Proxy through Cloudflare (orange cloud)")
                                .action(ArgAction::SetTrue),
                            Arg::new("ttl")
                                .long("ttl")
                                .help("TTL in seconds (1 = automatic)")
                                .default_value("1"),
                        ]),
                )
                .subcommand(
                    Command::new("rm")
                        .alias("remove")
                        .about("Remove a DNS record")
                        .long_about("Remove a DNS record by ID.\nUse `dns ls` to find the record ID.")
                        .args(&[
                            Arg::new("id")
                                .help("DNS record ID to remove")
                                .required(true),
                            Arg::new("domain")
                                .help("Domain the record belongs to")
                                .required(true),
                            Arg::new("yes")
                                .long("yes")
                                .help("Skip confirmation prompt")
                                .action(ArgAction::SetTrue),
                        ]),
                ),
        )
        .subcommand(
            Command::new("completions")
                .about("Generate shell completion scripts")
                .arg_required_else_help(true)
                .arg(
                    Arg::new("shell")
                        .help("Shell type")
                        .required(true)
                        .value_parser(["bash", "zsh", "fish", "powershell", "elvish"]),
                ),
        )
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    let matches = build_app().get_matches();

    match matches.subcommand() {
        Some(("config", subs)) => match subs.subcommand() {
            Some(("auth", _)) => commands::config::auth::auth().await,
            _ => Ok(()),
        },
        Some(("completions", subs)) => {
            let shell = match subs.get_one::<String>("shell").unwrap().as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                "powershell" => Shell::PowerShell,
                "elvish" => Shell::Elvish,
                _ => unreachable!(),
            };
            let mut cmd = build_app();
            generate(
                shell,
                &mut cmd,
                env!("CARGO_PKG_NAME"),
                &mut std::io::stdout(),
            );
            Ok(())
        }
        _ => {
            let user = get_valid_user().await?;
            let client = Client::new(
                Credentials::from(user),
                ClientConfig::default(),
                Environment::Production,
            )?;

            match matches.subcommand() {
                Some(("domains", subs)) => match subs.subcommand() {
                    Some(("ls", _)) => {
                        let zones = flary::spinner::with_spinner(
                            "Fetching domains",
                            commands::domains::list::call_api(&client, None),
                        )
                        .await?;
                        for zone in &zones {
                            println!("{}", &zone.name);
                        }
                        Ok(())
                    }
                    _ => Ok(()),
                },
                Some(("dns", subs)) => match subs.subcommand() {
                    Some(("ls", args)) => {
                        let name = args.get_one::<String>("name").unwrap();
                        let records = flary::spinner::with_spinner(
                            "Fetching DNS records",
                            commands::dns::list::call_api(&client, name),
                        )
                        .await?;
                        commands::dns::list::print_records(&records);
                        Ok(())
                    }
                    Some(("add", args)) => {
                        let domain = args.get_one::<String>("domain").unwrap();
                        let record_name = args.get_one::<String>("name").unwrap();
                        let record_type = args.get_one::<String>("type").unwrap();
                        let value = args.get_one::<String>("value").unwrap();
                        let proxied = args.get_flag("proxied");
                        let ttl: u32 = args.get_one::<String>("ttl").unwrap().parse().unwrap_or(1);
                        let priority: Option<u16> = args
                            .get_one::<String>("priority")
                            .and_then(|p| p.parse().ok());

                        let record = flary::spinner::with_spinner(
                            "Adding DNS record",
                            commands::dns::add::call_api(
                                &client,
                                domain,
                                record_name,
                                record_type,
                                value,
                                proxied,
                                ttl,
                                priority,
                            ),
                        )
                        .await?;

                        println!(
                            "Created DNS record: {} {} {} (ID: {})",
                            record.name, record_type, value, record.id,
                        );
                        Ok(())
                    }
                    Some(("update", args)) => {
                        let id = args.get_one::<String>("id").unwrap();
                        let domain = args.get_one::<String>("domain").unwrap();
                        let record_name = args.get_one::<String>("name").unwrap();
                        let record_type = args.get_one::<String>("type").unwrap();
                        let value = args.get_one::<String>("value").unwrap();
                        let proxied = args.get_flag("proxied");
                        let ttl: u32 = args.get_one::<String>("ttl").unwrap().parse().unwrap_or(1);

                        let record = flary::spinner::with_spinner(
                            "Updating DNS record",
                            commands::dns::update::call_api(
                                &client,
                                id,
                                domain,
                                record_name,
                                record_type,
                                value,
                                proxied,
                                ttl,
                            ),
                        )
                        .await?;

                        println!(
                            "Updated DNS record: {} {} {} (ID: {})",
                            record.name, record_type, value, record.id,
                        );
                        Ok(())
                    }
                    Some(("rm", args)) => {
                        let id = args.get_one::<String>("id").unwrap();
                        let domain = args.get_one::<String>("domain").unwrap();
                        let yes = args.get_flag("yes");

                        commands::dns::rm(&client, id, domain, yes).await
                    }
                    _ => Ok(()),
                },
                _ => Ok(()),
            }
        }
    }
}
