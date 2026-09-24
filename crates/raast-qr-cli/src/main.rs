use clap::{Args, Parser, Subcommand, ValueEnum};
use raast_qr::{Fee, InitiationMethod, RaastQr};
use rust_decimal::Decimal;

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Human,
    Json,
}

#[derive(Parser)]
#[command(name = "raast-qr-cli")]
#[command(about = "High-performance SBP Raast & EMVCo QR CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify the structure and CRC16 checksum of an EMVCo QR payload
    Verify {
        /// Raw EMVCo payload string
        payload: String,

        /// MAI scheme GUID to select (default "pk.raast")
        #[arg(long, default_value = "pk.raast")]
        scheme_guid: String,
    },
    /// Decode and display supported fields from an EMVCo QR payload
    Decode {
        /// Raw EMVCo payload string
        payload: String,

        /// MAI scheme GUID to select (default "pk.raast")
        #[arg(long, default_value = "pk.raast")]
        scheme_guid: String,

        /// Output format (human readable or json)
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Human)]
        format: OutputFormat,
    },
    /// Generate a Raast-style EMVCo QR payload
    Generate(Box<GenerateArgs>),
}

#[derive(Args)]
struct GenerateArgs {
    /// Merchant Raast Identifier (Mobile Alias or IBAN)
    #[arg(short, long)]
    alias: String,

    /// Merchant Name (max 25 bytes)
    #[arg(short, long)]
    name: String,

    /// Merchant City (max 15 bytes)
    #[arg(short, long)]
    city: String,

    /// Merchant Account Information Tag (default "26", range 26..=51)
    #[arg(long, default_value = "26")]
    mai_tag: String,

    /// Scheme Identifier / GUID (default "pk.raast")
    #[arg(long, default_value = "pk.raast")]
    scheme_guid: String,

    /// Bank / Participant Code (MAI sub-tag 02)
    #[arg(long)]
    bank_code: Option<String>,

    /// Transaction Amount in PKR (dynamic payment)
    #[arg(long)]
    amount: Option<Decimal>,

    /// Prompt the payer for an optional tip (EMVCo Tag 55=01)
    #[arg(long, conflicts_with_all = ["fixed_fee", "percentage_fee"])]
    prompt_tip: bool,

    /// Merchant-defined fixed convenience fee in PKR (not a bank transfer fee)
    #[arg(long, conflicts_with = "percentage_fee")]
    fixed_fee: Option<Decimal>,

    /// Merchant-defined convenience fee percentage (0.01..=99.99)
    #[arg(long)]
    percentage_fee: Option<Decimal>,

    /// Merchant Category Code (4 digits, default "0000")
    #[arg(long, default_value = "0000")]
    mcc: String,

    /// Optional Bill / Invoice reference
    #[arg(short, long)]
    reference: Option<String>,

    /// Mark QR as dynamic (single-use with fixed amount)
    #[arg(long)]
    dynamic: bool,
}

fn format_fee(fee: Fee) -> String {
    match fee {
        Fee::PromptTip => "payer-entered tip".to_string(),
        Fee::Fixed(value) => format!("fixed convenience fee: {} PKR", value),
        Fee::Percentage(value) => format!("convenience fee: {}%", value),
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Verify {
            payload,
            scheme_guid,
        } => match RaastQr::parse_with_guid(&payload, &scheme_guid) {
            Ok(qr) => {
                println!("✓ VALID EMVCo QR structure and CRC (not payee authentication)");
                println!("  Merchant: {}", qr.merchant_name);
                println!("  Account ID: {}", qr.raast_id);
                println!("  City:     {}", qr.merchant_city);
                if let Some(amt) = qr.amount {
                    println!("  Base amount: {} PKR", amt);
                }
                if let Some(fee) = qr.fee {
                    println!("  Extra:   {}", format_fee(fee));
                }
            }
            Err(e) => {
                eprintln!("✗ INVALID EMVCo QR payload: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Decode {
            payload,
            scheme_guid,
            format,
        } => match RaastQr::parse_with_guid(&payload, &scheme_guid) {
            Ok(qr) => match format {
                OutputFormat::Human => {
                    println!("=== EMVCo QR Payload ===");
                    println!("Initiation Method : {:?}", qr.initiation_method);
                    println!("MAI Tag           : {}", qr.mai_tag);
                    println!("Scheme GUID       : {}", qr.scheme_guid);
                    println!("MAI Account ID    : {}", qr.raast_id);
                    println!("Bank Code         : {:?}", qr.bank_code);
                    println!("Merchant Name     : {}", qr.merchant_name);
                    println!("Merchant City     : {}", qr.merchant_city);
                    println!("MCC               : {}", qr.mcc);
                    println!("Currency          : {:?}", qr.currency);
                    println!("Base Amount       : {:?}", qr.amount);
                    println!("Tip / Fee         : {:?}", qr.fee);
                    println!("Country Code      : {}", qr.country_code);
                    println!("Bill Reference    : {:?}", qr.bill_reference);
                }
                OutputFormat::Json => match serde_json::to_string_pretty(&qr) {
                    Ok(json) => println!("{}", json),
                    Err(e) => {
                        eprintln!("Failed to serialize decoded payload: {}", e);
                        std::process::exit(1);
                    }
                },
            },
            Err(e) => {
                eprintln!("Error decoding payload: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Generate(args) => {
            let GenerateArgs {
                alias,
                name,
                city,
                mai_tag,
                scheme_guid,
                bank_code,
                amount,
                prompt_tip,
                fixed_fee,
                percentage_fee,
                mcc,
                reference,
                dynamic,
            } = *args;
            let mut builder = RaastQr::builder()
                .mai_tag(mai_tag)
                .scheme_guid(scheme_guid)
                .raast_alias(alias)
                .merchant_name(name)
                .merchant_city(city)
                .mcc(mcc);

            if dynamic {
                builder = builder.initiation_method(InitiationMethod::Dynamic);
            }

            if let Some(amt) = amount {
                builder = builder.amount(amt);
            }

            if prompt_tip {
                builder = builder.fee(Fee::PromptTip);
            } else if let Some(value) = fixed_fee {
                builder = builder.fee(Fee::Fixed(value));
            } else if let Some(value) = percentage_fee {
                builder = builder.fee(Fee::Percentage(value));
            }

            if let Some(ref_id) = reference {
                builder = builder.bill_reference(ref_id);
            }

            if let Some(bank) = bank_code {
                builder = builder.bank_code(bank);
            }

            match builder.build_emv_string() {
                Ok(emv) => println!("{}", emv),
                Err(e) => {
                    eprintln!("Failed to generate Raast QR: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
