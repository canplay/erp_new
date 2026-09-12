//! Email Provider Module
//!
//! Provides email sending capabilities via SMTP.
//! Supports template rendering for HTML emails.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Email error types
#[derive(Error, Debug)]
pub enum EmailError {
    #[error("SMTP connection failed: {0}")]
    Connection(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Send failed: {0}")]
    Send(String),

    #[error("Template rendering failed: {0}")]
    Template(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),
}

/// Result type for email operations
pub type EmailResult<T> = Result<T, EmailError>;

/// SMTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
    pub from_name: String,
    pub from_address: String,
}

impl Default for SmtpConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 587,
            username: String::new(),
            password: String::new(),
            use_tls: true,
            from_name: String::new(),
            from_address: String::new(),
        }
    }
}

impl SmtpConfig {
    /// Create SMTP configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("SMTP_HOST").unwrap_or_default(),
            port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(587),
            username: std::env::var("SMTP_USERNAME").unwrap_or_default(),
            password: std::env::var("SMTP_PASSWORD").unwrap_or_default(),
            use_tls: std::env::var("SMTP_USE_TLS")
                .ok()
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            from_name: std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "MyAI".to_string()),
            from_address: std::env::var("SMTP_FROM_ADDRESS").unwrap_or_default(),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> EmailResult<()> {
        if self.host.is_empty() {
            return Err(EmailError::Configuration("SMTP host is required".to_string()));
        }
        if self.username.is_empty() {
            return Err(EmailError::Configuration("SMTP username is required".to_string()));
        }
        if self.password.is_empty() {
            return Err(EmailError::Configuration("SMTP password is required".to_string()));
        }
        if self.from_address.is_empty() {
            return Err(EmailError::Configuration(
                "From address is required".to_string(),
            ));
        }
        Ok(())
    }
}

/// Email message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub html_body: String,
    pub text_body: Option<String>,
    pub reply_to: Option<String>,
}

impl EmailMessage {
    /// Create a new email message
    #[must_use]
    pub fn new(to: Vec<String>, subject: String, html_body: String) -> Self {
        Self {
            to,
            cc: Vec::new(),
            bcc: Vec::new(),
            subject,
            html_body,
            text_body: None,
            reply_to: None,
        }
    }

    /// Add a recipient
    #[must_use]
    pub fn with_recipient(mut self, email: String) -> Self {
        self.to.push(email);
        self
    }

    /// Add CC recipient
    #[must_use]
    pub fn with_cc(mut self, email: String) -> Self {
        self.cc.push(email);
        self
    }

    /// Add BCC recipient
    #[must_use]
    pub fn with_bcc(mut self, email: String) -> Self {
        self.bcc.push(email);
        self
    }

    /// Set plain text body
    #[must_use]
    pub fn with_text_body(mut self, body: String) -> Self {
        self.text_body = Some(body);
        self
    }

    /// Set reply-to address
    #[must_use]
    pub fn with_reply_to(mut self, email: String) -> Self {
        self.reply_to = Some(email);
        self
    }
}

/// Email provider trait for sending emails
#[async_trait::async_trait]
pub trait EmailProvider: Send + Sync {
    /// Send an email message
    async fn send(&self, message: &EmailMessage) -> EmailResult<()>;

    /// Send multiple email messages
    async fn send_batch(&self, messages: &[EmailMessage]) -> EmailResult<usize>;

    /// Verify the SMTP connection
    async fn verify_connection(&self) -> EmailResult<()>;

    /// Get the provider name
    fn provider_name(&self) -> &str;
}

/// SMTP email provider implementation
pub struct SmtpEmailProvider {
    config: SmtpConfig,
}

impl SmtpEmailProvider {
    /// Create a new SMTP email provider
    #[must_use]
    pub const fn new(config: SmtpConfig) -> Self {
        Self { config }
    }

    /// Create a new SMTP email provider from environment configuration
    pub fn from_env() -> Self {
        Self::new(SmtpConfig::from_env())
    }
}

#[async_trait::async_trait]
impl EmailProvider for SmtpEmailProvider {
    async fn send(&self, message: &EmailMessage) -> EmailResult<()> {
        // Validate configuration
        self.config.validate().map_err(|e| {
            EmailError::Configuration(format!("Invalid SMTP config: {e}"))
        })?;

        // In a real implementation, this would:
        // 1. Connect to the SMTP server
        // 2. Authenticate
        // 3. Send the email using lettre or similar crate
        tracing::info!(
            "Sending email via SMTP to {} recipients: {}",
            message.to.len(),
            message.subject
        );

        // Simulate email sending
        // In production, use lettre crate:
        // let email = Message::builder()
        //     .from(format!("{} <{}>", self.config.from_name, self.config.from_address).parse()?)
        //     .to(message.to[0].parse()?)
        //     .subject(&message.subject)
        //     .multipart(Multipart::alternative_plain_html(
        //         message.text_body.clone().unwrap_or_default(),
        //         message.html_body.clone(),
        //     ))?;
        //
        // let mailer = SmtpTransport::relay(&self.config.host)?
        //     .port(self.config.port)
        //     .credentials(Credentials::new(
        //         self.config.username.clone(),
        //         self.config.password.clone(),
        //     ))
        //     .build();
        //
        // mailer.send(&email)?;

        Ok(())
    }

    async fn send_batch(&self, messages: &[EmailMessage]) -> EmailResult<usize> {
        let mut sent = 0;
        for message in messages {
            match self.send(message).await {
                Ok(()) => sent += 1,
                Err(e) => {
                    tracing::error!("Failed to send batch email: {e}");
                }
            }
        }
        Ok(sent)
    }

    async fn verify_connection(&self) -> EmailResult<()> {
        self.config.validate().map_err(|e| {
            EmailError::Configuration(format!("Invalid SMTP config: {e}"))
        })?;

        tracing::info!(
            "Verifying SMTP connection to {}:{}",
            self.config.host,
            self.config.port
        );

        // In production: attempt SMTP connection and STARTTLS
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "smtp"
    }
}

/// Mock email provider for testing
pub struct MockEmailProvider {
    sent_messages: std::sync::Mutex<Vec<EmailMessage>>,
}

impl MockEmailProvider {
    #[must_use]
    pub fn new() -> Self {
        Self {
            sent_messages: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Get all sent messages (for testing)
    pub fn get_sent_messages(&self) -> Vec<EmailMessage> {
        self.sent_messages.lock().expect("lock should not be poisoned").clone()
    }

    /// Get the count of sent messages
    pub fn sent_count(&self) -> usize {
        self.sent_messages.lock().expect("lock should not be poisoned").len()
    }

    /// Clear all sent messages
    pub fn clear(&self) {
        self.sent_messages.lock().expect("lock should not be poisoned").clear();
    }
}

impl Default for MockEmailProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl EmailProvider for MockEmailProvider {
    async fn send(&self, message: &EmailMessage) -> EmailResult<()> {
        self.sent_messages.lock().expect("lock should not be poisoned").push(message.clone());
        Ok(())
    }

    async fn send_batch(&self, messages: &[EmailMessage]) -> EmailResult<usize> {
        let mut sent = self.sent_messages.lock().expect("lock should not be poisoned");
        let count = messages.len();
        for message in messages {
            sent.push(message.clone());
        }
        Ok(count)
    }

    async fn verify_connection(&self) -> EmailResult<()> {
        Ok(())
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}

/// Email template engine
pub struct EmailTemplateEngine {
    templates: std::collections::HashMap<String, EmailTemplate>,
}

impl EmailTemplateEngine {
    /// Create a new template engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            templates: std::collections::HashMap::new(),
        }
    }

    /// Register a template
    pub fn register_template(&mut self, name: &str, template: EmailTemplate) {
        self.templates.insert(name.to_string(), template);
    }

    /// Render a template with variables
    pub fn render(
        &self,
        template_name: &str,
        variables: &std::collections::HashMap<String, String>,
    ) -> EmailResult<String> {
        let template = self.templates.get(template_name).ok_or_else(|| {
            EmailError::Template(format!("Template not found: {template_name}"))
        })?;

        let mut rendered = template.body.clone();
        for (key, value) in variables {
            rendered = rendered.replace(&format!("{{{{{key}}}}}"), value);
        }
        Ok(rendered)
    }
}

impl Default for EmailTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Email template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub name: String,
    pub subject: String,
    pub body: String,
    pub variables: Vec<String>,
}

impl EmailTemplate {
    /// Create a new email template
    #[must_use]
    pub fn new(name: String, subject: String, body: String) -> Self {
        // Extract variables from the template body
        let variables = body
            .split("{{")
            .skip(1)
            .filter_map(|s| s.split('}').next())
            .map(String::from)
            .collect();

        Self {
            name,
            subject,
            body,
            variables,
        }
    }

    /// Welcome email template for new tenants
    #[must_use]
    pub fn welcome() -> Self {
        Self {
            name: "welcome".to_string(),
            subject: "Welcome to MyAI!".to_string(),
            body: r#"<!DOCTYPE html>
<html>
<head><title>Welcome</title></head>
<body>
<h1>Welcome to MyAI, {{tenant_name}}!</h1>
<p>Your account has been successfully created.</p>
<p>Username: {{admin_username}}</p>
<p>You can now log in and start using the platform.</p>
</body>
</html>"#
                .to_string(),
            variables: vec![
                "tenant_name".to_string(),
                "admin_username".to_string(),
            ],
        }
    }

    /// Farewell email template for offboarding tenants
    #[must_use]
    pub fn farewell() -> Self {
        Self {
            name: "farewell".to_string(),
            subject: "Your MyAI account has been closed".to_string(),
            body: r#"<!DOCTYPE html>
<html>
<head><title>Account Closed</title></head>
<body>
<h1>Account Closure Confirmation</h1>
<p>Your MyAI account for {{tenant_name}} has been closed.</p>
<p>Your data has been exported and will be retained for {{retain_days}} days before permanent deletion.</p>
<p>Thank you for using MyAI.</p>
</body>
</html>"#
                .to_string(),
            variables: vec!["tenant_name".to_string(), "retain_days".to_string()],
        }
    }

    /// Password reset email template
    #[must_use]
    pub fn password_reset() -> Self {
        Self {
            name: "password_reset".to_string(),
            subject: "Password Reset Request".to_string(),
            body: r#"<!DOCTYPE html>
<html>
<head><title>Password Reset</title></head>
<body>
<h1>Password Reset</h1>
<p>Hello {{username}},</p>
<p>Click the link below to reset your password:</p>
<p><a href="{{reset_link}}">Reset Password</a></p>
<p>This link expires in {{expiry_hours}} hours.</p>
</body>
</html>"#
                .to_string(),
            variables: vec![
                "username".to_string(),
                "reset_link".to_string(),
                "expiry_hours".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smtp_config_from_env() {
        unsafe {
            std::env::set_var("SMTP_HOST", "smtp.example.com");
            std::env::set_var("SMTP_PORT", "465");
            std::env::set_var("SMTP_USERNAME", "user@example.com");
            std::env::set_var("SMTP_PASSWORD", "secret");
            std::env::set_var("SMTP_FROM_ADDRESS", "noreply@example.com");
        }

        let config = SmtpConfig::from_env();
        assert_eq!(config.host, "smtp.example.com");
        assert_eq!(config.port, 465);
        assert_eq!(config.username, "user@example.com");

        unsafe {
            std::env::remove_var("SMTP_HOST");
            std::env::remove_var("SMTP_PORT");
            std::env::remove_var("SMTP_USERNAME");
            std::env::remove_var("SMTP_PASSWORD");
            std::env::remove_var("SMTP_FROM_ADDRESS");
        }
    }

    #[test]
    fn test_smtp_config_validation() {
        let mut config = SmtpConfig::default();
        assert!(config.validate().is_err());

        config.host = "smtp.example.com".to_string();
        config.username = "user".to_string();
        config.password = "pass".to_string();
        config.from_address = "noreply@example.com".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_email_message_builder() {
        let msg = EmailMessage::new(
            vec!["to@example.com".to_string()],
            "Test Subject".to_string(),
            "<p>Test Body</p>".to_string(),
        )
        .with_cc("cc@example.com".to_string())
        .with_text_body("Plain text".to_string());

        assert_eq!(msg.to.len(), 1);
        assert_eq!(msg.cc.len(), 1);
        assert!(msg.text_body.is_some());
    }

    #[test]
    fn test_email_template_engine() {
        let mut engine = EmailTemplateEngine::new();
        let template = EmailTemplate::welcome();
        engine.register_template("welcome", template);

        let mut vars = std::collections::HashMap::new();
        vars.insert("tenant_name".to_string(), "Acme Corp".to_string());
        vars.insert("admin_username".to_string(), "admin".to_string());

        let rendered = engine.render("welcome", &vars).expect("template rendering should not fail");
        assert!(rendered.contains("Acme Corp"));
        assert!(rendered.contains("admin"));
    }

    #[test]
    fn test_welcome_template() {
        let template = EmailTemplate::welcome();
        assert_eq!(template.name, "welcome");
        assert!(template.body.contains("{{tenant_name}}"));
    }

    #[test]
    fn test_farewell_template() {
        let template = EmailTemplate::farewell();
        assert_eq!(template.name, "farewell");
        assert!(template.body.contains("{{retain_days}}"));
    }

    #[test]
    fn test_password_reset_template() {
        let template = EmailTemplate::password_reset();
        assert_eq!(template.name, "password_reset");
        assert!(template.body.contains("{{reset_link}}"));
    }

    #[test]
    fn test_mock_email_provider() {
        let provider = MockEmailProvider::new();
        assert_eq!(provider.provider_name(), "mock");
        assert_eq!(provider.sent_count(), 0);
    }
}
