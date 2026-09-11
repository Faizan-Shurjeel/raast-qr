use clap::{Parser, Subcommand};
use raast_qr::{InitiationMethod, RaastQr};
use rust_decimal::Decimal;

#[derive(Parser)]
#[command(name = "raast-qr")]
#[command(about = "High-performance SBP Raast & EMVCo QR CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Verify the integrity and CRC16 checksum of a Raast QR payload
    Verify {
        /// Raw EMVCo payload string
        payload: String,
    },
    /// Decode and display all fields from an EMVCo Raast QR payload
    Decode {
        /// Raw EMVCo payload string
        payload: String,
    },
    /// Generate a compliant SBP Raast EMVCo QR payload
    Generate {
        /// Merchant Raast Identifier (Mobile Alias or IBAN)
        #[arg(short, long)]
        alias: String,

        /// Merchant Name (max 25 characters)
        #[arg(short, long)]
        name: String,

        /// Merchant City (max 15 characters)
        #[arg(short, long)]
        city: String,

        /// Transaction Amount in PKR (dynamic payment)
        #[arg(long)]
        amount: Option<Decimal>,

        /// Merchant Category Code (4 digits, default "0000")
        #[arg(long, default_value = "0000")]
        mcc: String,

        /// Optional Bill / Invoice reference
        #[arg(short, long)]
        reference: Option<String>,

        /// Mark QR as dynamic (single-use with fixed amount)
        #[arg(long)]
        dynamic: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Verify { payload } => match RaastQr::parse(&payload) {
            Ok(qr) => {
                println!("✓ VALID SBP Raast QR Code");
                println!("  Merchant: {}", qr.merchant_name);
                println!("  Raast ID: {}", qr.raast_id);
                println!("  City:     {}", qr.merchant_city);
                if let Some(amt) = qr.amount {
                    println!("  Amount:   {} PKR", amt);
                }
            }
            Err(e) => {
                eprintln!("✗ INVALID SBP Raast QR Code: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Decode { payload } => match RaastQr::parse(&payload) {
            Ok(qr) => {
                println!("=== SBP Raast QR Payload ===");
                println!("Initiation Method : {:?}", qr.initiation_method);
                println!("Raast ID / Alias  : {}", qr.raast_id);
                println!("Bank Code         : {:?}", qr.bank_code);
                println!("Merchant Name     : {}", qr.merchant_name);
                println!("Merchant City     : {}", qr.merchant_city);
                println!("MCC               : {}", qr.mcc);
                println!("Currency          : {:?}", qr.currency);
                println!("Amount            : {:?}", qr.amount);
                println!("Country Code      : {}", qr.country_code);
                println!("Bill Reference    : {:?}", qr.bill_reference);
            }
            Err(e) => {
                eprintln!("Error decoding payload: {}", e);
                std::process::exit(1);
            }
        },
        Commands::Generate {
            alias,
            name,
            city,
            amount,
            mcc,
            reference,
            dynamic,
        } => {
            let mut builder = RaastQr::builder()
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

            if let Some(ref_id) = reference {
                builder = builder.bill_reference(ref_id);
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
