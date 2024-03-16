use lettre::{transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use once_cell::sync::Lazy;

use crate::parse_env_var;

const SENDER_EMAIL: Lazy<String> = Lazy::new(|| parse_env_var("SMTP_USERNAME"));
const SENDER_PASSWORD: Lazy<String> = Lazy::new(|| parse_env_var("SMTP_PASSWORD"));
const SMTP_SERVER: Lazy<String> = Lazy::new(|| parse_env_var("SMTP_SERVER"));

pub fn send_mail(receiver_email: String, title: String, body: String) -> Option<()> {
    let email = Message::builder() 
        .from(format!("Sender <{}>", SENDER_EMAIL.to_owned()).parse().ok()?) 
        .to(format!("Receiver <{}>", receiver_email).parse().ok()?) 
        .subject(title) 
        .body(body)
        .ok()?; 

    let creds = Credentials::new(SENDER_EMAIL.to_owned(), SENDER_PASSWORD.to_string()); 

    // Open a remote connection to gmail 
    let mailer = SmtpTransport::relay(SMTP_SERVER.as_str()) 
    .unwrap() 
    .credentials(creds) 
    .build(); 

    // Send the email 
    match mailer.send(&email) { 
        Ok(_) => Some(()), 
        Err(_) => None, 
    }
}