//! PDF generation service for receipts and labels.
//!
//! Generates PDF documents for customer receipts and physical labels.

use crate::error::AppError;
use crate::models::ticket::Ticket;
use crate::models::Customer;
use printpdf::*;
use std::io::BufWriter;

/// Label data for PDF generation.
pub struct LabelData {
    pub ticket: Ticket,
    pub customer_name: String,
}

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

/// Generate a label PDF for a physical tag.
///
/// The label is 2x1 inches (50.8mm x 25.4mm) and includes:
/// - Ticket friendly code (large, prominent)
/// - Short item descriptor
///
/// This is designed to print on standard jewelry tag stock.
pub fn generate_label_pdf(data: &LabelData) -> Result<Vec<u8>, AppError> {
    // Create PDF document
    // Label size: 2" x 1" = 50.8mm x 25.4mm
    let (doc, page1, layer1) = PdfDocument::new("Repair Label", Mm(50.8), Mm(25.4), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Load built-in fonts
    let font_bold = doc
        .add_builtin_font(BuiltinFont::HelveticaBold)
        .map_err(|e| AppError::server_error(format!("Failed to load bold font: {:?}", e)))?;
    let font = doc
        .add_builtin_font(BuiltinFont::Helvetica)
        .map_err(|e| AppError::server_error(format!("Failed to load font: {:?}", e)))?;

    // Label layout - centered content with small margins
    // Y starts from top (25.4mm height)
    let center_x = 25.4; // Center of the label width
    let margin = 2.0;

    // === Ticket Code (large, prominent, centered) ===
    // Place near top of label
    let code_y = 20.0;
    let code_text = &data.ticket.friendly_code;

    // Calculate approximate text width for centering (rough estimate: 3.5mm per char at size 14)
    let code_width_estimate = code_text.len() as f32 * 3.5;
    let code_x = center_x - (code_width_estimate / 2.0);

    current_layer.use_text(
        code_text,
        14.0,
        Mm(code_x.max(margin)),
        Mm(code_y),
        &font_bold,
    );

    // === Customer Name (below ticket code) ===
    // Truncate long names to fit on small label
    let customer_name = truncate_text(&data.customer_name, 20);
    let name_y = 14.5;
    let name_width_estimate = customer_name.len() as f32 * 2.0; // Approx 2mm per char at size 9
    let name_x = center_x - (name_width_estimate / 2.0);

    current_layer.use_text(
        &customer_name,
        9.0,
        Mm(name_x.max(margin)),
        Mm(name_y),
        &font,
    );

    // === Item Descriptor (smaller, below customer name) ===
    // Create a short descriptor from item_type and truncated description
    let descriptor = create_short_descriptor(
        data.ticket.item_type.as_deref(),
        &data.ticket.item_description,
    );

    // Smaller font for descriptor
    let desc_y = 9.0;
    let desc_width_estimate = descriptor.len() as f32 * 1.8; // Approx 1.8mm per char at size 8
    let desc_x = center_x - (desc_width_estimate / 2.0);

    current_layer.use_text(&descriptor, 8.0, Mm(desc_x.max(margin)), Mm(desc_y), &font);

    // === Rush indicator (if applicable) ===
    if data.ticket.is_rush {
        let rush_y = 3.5;
        let rush_text = "RUSH";
        let rush_width_estimate = rush_text.len() as f32 * 2.5;
        let rush_x = center_x - (rush_width_estimate / 2.0);
        current_layer.use_text(
            rush_text,
            10.0,
            Mm(rush_x.max(margin)),
            Mm(rush_y),
            &font_bold,
        );
    }

    // Save PDF to bytes
    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)
        .map_err(|e| AppError::server_error(format!("Failed to save PDF: {:?}", e)))?;

    buffer
        .into_inner()
        .map_err(|e| AppError::server_error(format!("Failed to get PDF buffer: {:?}", e)))
}

/// Create a short descriptor for the label from item type and description.
///
/// Combines item_type (if present) with a truncated description,
/// keeping the total length reasonable for a small label.
fn create_short_descriptor(item_type: Option<&str>, description: &str) -> String {
    let max_len: usize = 25; // Maximum characters for the descriptor

    match item_type {
        Some(t) if !t.is_empty() => {
            // "Ring - Gold band with..."
            let prefix = format!("{} - ", t);
            let remaining = max_len.saturating_sub(prefix.len());

            if remaining > 3 {
                let truncated_desc = truncate_text(description, remaining);
                format!("{}{}", prefix, truncated_desc)
            } else {
                truncate_text(t, max_len)
            }
        }
        _ => truncate_text(description, max_len),
    }
}

/// Truncate text to a maximum length, adding "..." if truncated.
fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else if max_len <= 3 {
        text.chars().take(max_len).collect()
    } else {
        let truncated: String = text.chars().take(max_len - 3).collect();
        format!("{}...", truncated.trim_end())
    }
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

    #[test]
    fn test_truncate_text_short() {
        let result = truncate_text("Hello", 10);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_truncate_text_exact() {
        let result = truncate_text("Hello", 5);
        assert_eq!(result, "Hello");
    }

    #[test]
    fn test_truncate_text_long() {
        let result = truncate_text("Hello World", 8);
        assert_eq!(result, "Hello...");
    }

    #[test]
    fn test_truncate_text_very_short_max() {
        let result = truncate_text("Hello", 3);
        assert_eq!(result, "Hel");
    }

    #[test]
    fn test_create_short_descriptor_with_type() {
        let result = create_short_descriptor(Some("Ring"), "Gold band with diamonds");
        assert_eq!(result, "Ring - Gold band with...");
    }

    #[test]
    fn test_create_short_descriptor_without_type() {
        let result = create_short_descriptor(None, "Gold band with diamonds");
        assert_eq!(result, "Gold band with diamonds");
    }

    #[test]
    fn test_create_short_descriptor_empty_type() {
        let result = create_short_descriptor(Some(""), "Gold band");
        assert_eq!(result, "Gold band");
    }

    #[test]
    fn test_create_short_descriptor_short_description() {
        let result = create_short_descriptor(Some("Ring"), "Gold");
        assert_eq!(result, "Ring - Gold");
    }

    #[test]
    fn test_create_short_descriptor_long_description() {
        let result = create_short_descriptor(
            Some("Necklace"),
            "Beautiful platinum chain with sapphire pendant",
        );
        assert!(result.len() <= 25);
        assert!(result.ends_with("..."));
    }
}
