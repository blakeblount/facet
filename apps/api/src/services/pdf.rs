//! PDF generation service for receipts and labels.
//!
//! Generates PDF documents for customer receipts and physical labels.

use crate::error::AppError;
use crate::models::ticket::Ticket;
use crate::models::Customer;
use printpdf::*;
use std::io::BufWriter;

/// Receipt data for PDF generation.
pub struct ReceiptData {
    pub ticket: Ticket,
    pub customer: Customer,
    pub store_name: String,
    pub store_phone: Option<String>,
    pub store_address: Option<String>,
}

/// Generate a receipt PDF for a ticket.
///
/// The receipt includes:
/// - Ticket friendly code
/// - Customer name and contact info
/// - Item description and condition
/// - Requested work
/// - Quote amount and promise date
/// - Store information
pub fn generate_receipt_pdf(data: &ReceiptData) -> Result<Vec<u8>, AppError> {
    // Create PDF document
    // Letter size: 8.5" x 11" = 215.9mm x 279.4mm
    let (doc, page1, layer1) = PdfDocument::new("Repair Receipt", Mm(215.9), Mm(279.4), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Load built-in font
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| AppError::server_error(format!("Failed to load font: {:?}", e)))?;
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| AppError::server_error(format!("Failed to load bold font: {:?}", e)))?;

    let mut y_pos = 260.0; // Start from top (with margin)
    let left_margin = 20.0;
    let line_height = 6.0;
    let section_gap = 10.0;

    // === Store Header ===
    current_layer.use_text(
        &data.store_name,
        18.0,
        Mm(left_margin),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= line_height * 1.5;

    if let Some(ref phone) = data.store_phone {
        current_layer.use_text(phone, 10.0, Mm(left_margin), Mm(y_pos), &font);
        y_pos -= line_height;
    }

    if let Some(ref address) = data.store_address {
        current_layer.use_text(address, 10.0, Mm(left_margin), Mm(y_pos), &font);
        y_pos -= line_height;
    }

    y_pos -= section_gap;

    // === Repair Receipt Title ===
    current_layer.use_text(
        "REPAIR RECEIPT",
        14.0,
        Mm(left_margin),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= section_gap;

    // === Ticket ID (prominent) ===
    current_layer.use_text(
        format!("Ticket #: {}", data.ticket.friendly_code),
        16.0,
        Mm(left_margin),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= line_height * 2.0;

    // === Customer Info ===
    current_layer.use_text("CUSTOMER", 10.0, Mm(left_margin), Mm(y_pos), &font_bold);
    y_pos -= line_height;

    current_layer.use_text(&data.customer.name, 12.0, Mm(left_margin), Mm(y_pos), &font);
    y_pos -= line_height;

    if let Some(ref phone) = data.customer.phone {
        current_layer.use_text(
            format!("Phone: {}", phone),
            10.0,
            Mm(left_margin),
            Mm(y_pos),
            &font,
        );
        y_pos -= line_height;
    }

    if let Some(ref email) = data.customer.email {
        current_layer.use_text(
            format!("Email: {}", email),
            10.0,
            Mm(left_margin),
            Mm(y_pos),
            &font,
        );
        y_pos -= line_height;
    }

    y_pos -= section_gap;

    // === Item Details ===
    current_layer.use_text("ITEM DETAILS", 10.0, Mm(left_margin), Mm(y_pos), &font_bold);
    y_pos -= line_height;

    if let Some(ref item_type) = data.ticket.item_type {
        current_layer.use_text(
            format!("Type: {}", item_type),
            10.0,
            Mm(left_margin),
            Mm(y_pos),
            &font,
        );
        y_pos -= line_height;
    }

    // Word wrap long descriptions
    let desc_lines = wrap_text(&data.ticket.item_description, 80);
    current_layer.use_text("Description:", 10.0, Mm(left_margin), Mm(y_pos), &font);
    y_pos -= line_height;
    for line in desc_lines {
        current_layer.use_text(&line, 10.0, Mm(left_margin + 5.0), Mm(y_pos), &font);
        y_pos -= line_height;
    }

    let condition_lines = wrap_text(&data.ticket.condition_notes, 80);
    current_layer.use_text("Condition:", 10.0, Mm(left_margin), Mm(y_pos), &font);
    y_pos -= line_height;
    for line in condition_lines {
        current_layer.use_text(&line, 10.0, Mm(left_margin + 5.0), Mm(y_pos), &font);
        y_pos -= line_height;
    }

    y_pos -= section_gap;

    // === Requested Work ===
    current_layer.use_text(
        "REQUESTED WORK",
        10.0,
        Mm(left_margin),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= line_height;

    let work_lines = wrap_text(&data.ticket.requested_work, 80);
    for line in work_lines {
        current_layer.use_text(&line, 10.0, Mm(left_margin), Mm(y_pos), &font);
        y_pos -= line_height;
    }

    y_pos -= section_gap;

    // === Pricing & Dates ===
    if let Some(quote) = data.ticket.quote_amount {
        current_layer.use_text(
            format!("Estimated Price: ${:.2}", quote),
            12.0,
            Mm(left_margin),
            Mm(y_pos),
            &font_bold,
        );
        y_pos -= line_height * 1.5;
    }

    if let Some(promise_date) = data.ticket.promise_date {
        current_layer.use_text(
            format!("Promise Date: {}", promise_date.format("%B %d, %Y")),
            10.0,
            Mm(left_margin),
            Mm(y_pos),
            &font,
        );
        y_pos -= line_height;
    }

    if data.ticket.is_rush {
        current_layer.use_text(
            "*** RUSH ORDER ***",
            12.0,
            Mm(left_margin),
            Mm(y_pos),
            &font_bold,
        );
        y_pos -= line_height * 1.5;
    }

    y_pos -= section_gap;

    // === Date & Signature ===
    let created_date = data.ticket.created_at.format("%B %d, %Y at %I:%M %p");
    current_layer.use_text(
        format!("Date Received: {}", created_date),
        10.0,
        Mm(left_margin),
        Mm(y_pos),
        &font,
    );
    y_pos -= line_height * 3.0;

    // Signature line
    current_layer.use_text(
        "Customer Signature: ____________________________",
        10.0,
        Mm(left_margin),
        Mm(y_pos),
        &font,
    );
    y_pos -= line_height * 3.0;

    // === Footer ===
    current_layer.use_text(
        "Please retain this receipt for pickup.",
        9.0,
        Mm(left_margin),
        Mm(y_pos),
        &font,
    );
    y_pos -= line_height;
    current_layer.use_text(
        format!(
            "Ticket ID required for all inquiries: {}",
            data.ticket.friendly_code
        ),
        9.0,
        Mm(left_margin),
        Mm(y_pos),
        &font,
    );

    // Suppress unused variable warning for final y_pos
    let _ = y_pos;

    // Save PDF to bytes
    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)
        .map_err(|e| AppError::server_error(format!("Failed to save PDF: {:?}", e)))?;

    buffer
        .into_inner()
        .map_err(|e| AppError::server_error(format!("Failed to get PDF buffer: {:?}", e)))
}

/// Simple text wrapper for PDF output.
fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_text_short() {
        let lines = wrap_text("Hello world", 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "Hello world");
    }

    #[test]
    fn test_wrap_text_long() {
        let text = "This is a longer text that should be wrapped across multiple lines when it exceeds the maximum character limit";
        let lines = wrap_text(text, 40);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(line.len() <= 45); // Some tolerance for word boundaries
        }
    }

    #[test]
    fn test_wrap_text_empty() {
        let lines = wrap_text("", 80);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "");
    }
}
