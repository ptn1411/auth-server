use lettre::{
    message::{header::ContentType, Mailbox},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use std::sync::Arc;
use tracing::{error, info};

use crate::error::AuthError;

/// Email configuration
#[derive(Clone, Debug)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub app_name: String,
    pub app_url: String,
}

impl EmailConfig {
    pub fn from_env() -> Option<Self> {
        let smtp_host = std::env::var("SMTP_HOST").ok()?;
        let smtp_port = std::env::var("SMTP_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(587);
        let smtp_username = std::env::var("SMTP_USERNAME").ok()?;
        let smtp_password = std::env::var("SMTP_PASSWORD").ok()?;
        let from_email = std::env::var("SMTP_FROM_EMAIL").ok()?;
        let from_name = std::env::var("SMTP_FROM_NAME").unwrap_or_else(|_| "Auth Server".to_string());
        let app_name = std::env::var("APP_NAME").unwrap_or_else(|_| "Auth Server".to_string());
        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        Some(Self {
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            from_email,
            from_name,
            app_name,
            app_url,
        })
    }
}

/// Email service for sending transactional emails
#[derive(Clone)]
pub struct EmailService {
    config: Arc<EmailConfig>,
    mailer: Arc<AsyncSmtpTransport<Tokio1Executor>>,
}

impl EmailService {
    /// Create a new email service
    pub fn new(config: EmailConfig) -> Result<Self, AuthError> {
        let creds = Credentials::new(config.smtp_username.clone(), config.smtp_password.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
            .map_err(|e| AuthError::InternalError(e.into()))?
            .port(config.smtp_port)
            .credentials(creds)
            .build();

        Ok(Self {
            config: Arc::new(config),
            mailer: Arc::new(mailer),
        })
    }

    /// Send an email
    async fn send_email(&self, to: &str, subject: &str, html_body: &str) -> Result<(), AuthError> {
        let from: Mailbox = format!("{} <{}>", self.config.from_name, self.config.from_email)
            .parse()
            .map_err(|e: lettre::address::AddressError| AuthError::InternalError(e.into()))?;

        let to_mailbox: Mailbox = to
            .parse()
            .map_err(|e: lettre::address::AddressError| AuthError::InternalError(e.into()))?;

        let email = Message::builder()
            .from(from)
            .to(to_mailbox)
            .subject(subject)
            .header(ContentType::TEXT_HTML)
            .body(html_body.to_string())
            .map_err(|e| AuthError::InternalError(e.into()))?;

        self.mailer
            .send(email)
            .await
            .map_err(|e| AuthError::InternalError(e.into()))?;

        info!("Email sent to {}", to);
        Ok(())
    }

    /// Send password reset email
    pub async fn send_password_reset(&self, to: &str, reset_token: &str) -> Result<(), AuthError> {
        let reset_url = format!("{}/reset-password?token={}", self.config.app_url, reset_token);
        
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #4F46E5; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 30px; background: #f9fafb; }}
        .button {{ display: inline-block; padding: 12px 24px; background: #4F46E5; color: white; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ padding: 20px; text-align: center; color: #666; font-size: 12px; }}
        .warning {{ color: #dc2626; font-size: 14px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{app_name}</h1>
        </div>
        <div class="content">
            <h2>Password Reset Request</h2>
            <p>We received a request to reset your password. Click the button below to create a new password:</p>
            <p style="text-align: center;">
                <a href="{reset_url}" class="button">Reset Password</a>
            </p>
            <p>Or copy and paste this link into your browser:</p>
            <p style="word-break: break-all; color: #4F46E5;">{reset_url}</p>
            <p class="warning">This link will expire in 1 hour.</p>
            <p>If you didn't request a password reset, you can safely ignore this email.</p>
        </div>
        <div class="footer">
            <p>© {year} {app_name}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
"#,
            app_name = self.config.app_name,
            reset_url = reset_url,
            year = chrono::Utc::now().format("%Y")
        );

        self.send_email(to, &format!("Reset your {} password", self.config.app_name), &html).await
    }

    /// Send email verification email
    pub async fn send_email_verification(&self, to: &str, verification_token: &str) -> Result<(), AuthError> {
        let verify_url = format!("{}/verify-email?token={}", self.config.app_url, verification_token);
        
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #4F46E5; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 30px; background: #f9fafb; }}
        .button {{ display: inline-block; padding: 12px 24px; background: #4F46E5; color: white; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ padding: 20px; text-align: center; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{app_name}</h1>
        </div>
        <div class="content">
            <h2>Verify Your Email</h2>
            <p>Welcome to {app_name}! Please verify your email address by clicking the button below:</p>
            <p style="text-align: center;">
                <a href="{verify_url}" class="button">Verify Email</a>
            </p>
            <p>Or copy and paste this link into your browser:</p>
            <p style="word-break: break-all; color: #4F46E5;">{verify_url}</p>
            <p>This link will expire in 24 hours.</p>
        </div>
        <div class="footer">
            <p>© {year} {app_name}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
"#,
            app_name = self.config.app_name,
            verify_url = verify_url,
            year = chrono::Utc::now().format("%Y")
        );

        self.send_email(to, &format!("Verify your {} email", self.config.app_name), &html).await
    }

    /// Send welcome email after registration
    pub async fn send_welcome(&self, to: &str, user_name: Option<&str>) -> Result<(), AuthError> {
        let name = user_name.unwrap_or("there");
        
        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #4F46E5; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 30px; background: #f9fafb; }}
        .button {{ display: inline-block; padding: 12px 24px; background: #4F46E5; color: white; text-decoration: none; border-radius: 6px; margin: 20px 0; }}
        .footer {{ padding: 20px; text-align: center; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Welcome to {app_name}!</h1>
        </div>
        <div class="content">
            <h2>Hi {name}!</h2>
            <p>Thank you for joining {app_name}. We're excited to have you on board!</p>
            <p>Your account has been created successfully. You can now:</p>
            <ul>
                <li>Log in to your account</li>
                <li>Set up two-factor authentication for extra security</li>
                <li>Manage your profile settings</li>
            </ul>
            <p style="text-align: center;">
                <a href="{app_url}/login" class="button">Get Started</a>
            </p>
        </div>
        <div class="footer">
            <p>© {year} {app_name}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
"#,
            app_name = self.config.app_name,
            app_url = self.config.app_url,
            name = name,
            year = chrono::Utc::now().format("%Y")
        );

        self.send_email(to, &format!("Welcome to {}!", self.config.app_name), &html).await
    }

    /// Send security alert email (new login, password changed, etc.)
    pub async fn send_security_alert(
        &self,
        to: &str,
        alert_type: SecurityAlertType,
        details: Option<&str>,
    ) -> Result<(), AuthError> {
        let (title, message) = match alert_type {
            SecurityAlertType::NewLogin => (
                "New Login Detected",
                "A new login to your account was detected.",
            ),
            SecurityAlertType::PasswordChanged => (
                "Password Changed",
                "Your password was successfully changed.",
            ),
            SecurityAlertType::MfaEnabled => (
                "Two-Factor Authentication Enabled",
                "Two-factor authentication has been enabled on your account.",
            ),
            SecurityAlertType::MfaDisabled => (
                "Two-Factor Authentication Disabled",
                "Two-factor authentication has been disabled on your account.",
            ),
            SecurityAlertType::AccountLocked => (
                "Account Locked",
                "Your account has been temporarily locked due to multiple failed login attempts.",
            ),
            SecurityAlertType::SuspiciousActivity => (
                "Suspicious Activity Detected",
                "We detected suspicious activity on your account.",
            ),
        };

        let details_html = details
            .map(|d| format!("<p><strong>Details:</strong> {}</p>", d))
            .unwrap_or_default();

        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #dc2626; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 30px; background: #f9fafb; }}
        .alert {{ background: #fef2f2; border: 1px solid #fecaca; padding: 15px; border-radius: 6px; margin: 20px 0; }}
        .footer {{ padding: 20px; text-align: center; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🔒 Security Alert</h1>
        </div>
        <div class="content">
            <h2>{title}</h2>
            <div class="alert">
                <p>{message}</p>
                {details_html}
                <p><strong>Time:</strong> {time}</p>
            </div>
            <p>If this wasn't you, please secure your account immediately by:</p>
            <ul>
                <li>Changing your password</li>
                <li>Enabling two-factor authentication</li>
                <li>Reviewing your recent account activity</li>
            </ul>
            <p>If you recognize this activity, you can safely ignore this email.</p>
        </div>
        <div class="footer">
            <p>© {year} {app_name}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
"#,
            title = title,
            message = message,
            details_html = details_html,
            time = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            app_name = self.config.app_name,
            year = chrono::Utc::now().format("%Y")
        );

        self.send_email(to, &format!("[{}] {}", self.config.app_name, title), &html).await
    }

    /// Send MFA backup codes email
    pub async fn send_backup_codes(&self, to: &str, codes: &[String]) -> Result<(), AuthError> {
        let codes_html = codes
            .iter()
            .map(|c| format!("<li><code>{}</code></li>", c))
            .collect::<Vec<_>>()
            .join("\n");

        let html = format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body {{ font-family: Arial, sans-serif; line-height: 1.6; color: #333; }}
        .container {{ max-width: 600px; margin: 0 auto; padding: 20px; }}
        .header {{ background: #4F46E5; color: white; padding: 20px; text-align: center; }}
        .content {{ padding: 30px; background: #f9fafb; }}
        .codes {{ background: #fff; border: 1px solid #e5e7eb; padding: 20px; border-radius: 6px; margin: 20px 0; }}
        .codes ul {{ list-style: none; padding: 0; columns: 2; }}
        .codes li {{ padding: 5px 0; }}
        .codes code {{ background: #f3f4f6; padding: 4px 8px; border-radius: 4px; font-family: monospace; }}
        .warning {{ background: #fef3c7; border: 1px solid #fcd34d; padding: 15px; border-radius: 6px; margin: 20px 0; }}
        .footer {{ padding: 20px; text-align: center; color: #666; font-size: 12px; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>{app_name}</h1>
        </div>
        <div class="content">
            <h2>Your Backup Codes</h2>
            <p>Here are your two-factor authentication backup codes. Each code can only be used once.</p>
            <div class="codes">
                <ul>
                    {codes_html}
                </ul>
            </div>
            <div class="warning">
                <strong>⚠️ Important:</strong>
                <ul>
                    <li>Store these codes in a safe place</li>
                    <li>Each code can only be used once</li>
                    <li>Use these codes if you lose access to your authenticator app</li>
                    <li>Generate new codes if you run out or suspect they've been compromised</li>
                </ul>
            </div>
        </div>
        <div class="footer">
            <p>© {year} {app_name}. All rights reserved.</p>
        </div>
    </div>
</body>
</html>
"#,
            app_name = self.config.app_name,
            codes_html = codes_html,
            year = chrono::Utc::now().format("%Y")
        );

        self.send_email(to, &format!("[{}] Your Backup Codes", self.config.app_name), &html).await
    }
}

/// Types of security alerts
#[derive(Debug, Clone, Copy)]
pub enum SecurityAlertType {
    NewLogin,
    PasswordChanged,
    MfaEnabled,
    MfaDisabled,
    AccountLocked,
    SuspiciousActivity,
}

/// Mock email service for development/testing
#[derive(Clone)]
pub struct MockEmailService;

impl MockEmailService {
    pub fn new() -> Self {
        Self
    }

    pub async fn send_password_reset(&self, to: &str, reset_token: &str) -> Result<(), AuthError> {
        info!("[MOCK EMAIL] Password reset to {}: token={}", to, reset_token);
        Ok(())
    }

    pub async fn send_email_verification(&self, to: &str, verification_token: &str) -> Result<(), AuthError> {
        info!("[MOCK EMAIL] Email verification to {}: token={}", to, verification_token);
        Ok(())
    }

    pub async fn send_welcome(&self, to: &str, user_name: Option<&str>) -> Result<(), AuthError> {
        info!("[MOCK EMAIL] Welcome email to {}: name={:?}", to, user_name);
        Ok(())
    }

    pub async fn send_security_alert(
        &self,
        to: &str,
        alert_type: SecurityAlertType,
        details: Option<&str>,
    ) -> Result<(), AuthError> {
        info!("[MOCK EMAIL] Security alert to {}: type={:?}, details={:?}", to, alert_type, details);
        Ok(())
    }

    pub async fn send_backup_codes(&self, to: &str, codes: &[String]) -> Result<(), AuthError> {
        info!("[MOCK EMAIL] Backup codes to {}: {} codes", to, codes.len());
        Ok(())
    }
}

impl Default for MockEmailService {
    fn default() -> Self {
        Self::new()
    }
}
