use colored::Colorize;
use genpdf::{
    elements::{Break, FrameCellDecorator, Image, Paragraph, TableLayout},
    fonts::{FontData, FontFamily},
    style::{Color, Style},
    Alignment, Document, Element, PaperSize, SimplePageDecorator,
};
use printpdf;
use printpdf::types::plugins::graphics::two_dimensional::font::BuiltinFont;
use std::fs::read_dir;
fn build_document() -> Document {
    let builtin_font = Some(BuiltinFont::HelveticaBold);
    let load_helvetica_regular = include_bytes!("./assets/HelveticaRegular.ttf").to_vec();
    let load_helvetica_bold = include_bytes!("./assets/HelveticaBold.ttf").to_vec();
    let load_helvetica_italic = include_bytes!("./assets/HelveticaItalic.ttf").to_vec();
    let load_helvetica_bold_italic = include_bytes!("./assets/HelveticaBoldItalic.ttf").to_vec();

    let font_data_regular = FontData::new(load_helvetica_regular, builtin_font)
        .expect("Error while getting font_bytes\n");
    let font_data_bold =
        FontData::new(load_helvetica_bold, builtin_font).expect("Error while getting font_bytes\n");
    let font_data_italic = FontData::new(load_helvetica_italic, builtin_font)
        .expect("Error while getting font_bytes\n");
    let font_data_bold_italic = FontData::new(load_helvetica_bold_italic, builtin_font)
        .expect("Error while getting font_bytes\n");

    let font_family = FontFamily {
        regular: font_data_regular,
        bold: font_data_bold,
        italic: font_data_italic,
        bold_italic: font_data_bold_italic,
    };
    Document::new(font_family)
}
fn document_configuration(document: &mut Document, title: &str, page_title: &str) {
    document.set_title(title);
    document.set_minimal_conformance();
    document.set_line_spacing(1.25);
    document.push(
        Paragraph::new(page_title)
            .aligned(Alignment::Center)
            .styled(Style::new().bold()),
    );
    let mut page_decorator = SimplePageDecorator::default();
    page_decorator.set_margins(10);
    document.set_page_decorator(page_decorator);
    document.set_paper_size(PaperSize::Legal);
}
fn create_table(key: &str, value: &str) -> TableLayout {
    let mut table = TableLayout::new(vec![1, 1]);
    table.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    let row = table.row();
    row.element(
        Paragraph::new(key)
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_color(Color::Rgb(34, 91, 247))),
    )
    .element(
        Paragraph::new(value)
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_color(Color::Rgb(208, 97, 0))),
    )
    .push()
    .unwrap();
    table
        .row()
        .element(Break::new(1.0))
        .element(Break::new(1.0))
        .push()
        .unwrap();
    table
}
fn create_table_with_one_column(header: &str) -> TableLayout {
    let mut table = TableLayout::new(vec![1]);
    table.set_cell_decorator(FrameCellDecorator::new(true, true, false));
    let row = table.row();
    row.element(
        Paragraph::new(header)
            .aligned(Alignment::Center)
            .styled(Style::new().bold().with_color(Color::Rgb(34, 91, 247))),
    )
    .push()
    .unwrap();
    table.row().element(Break::new(1.0)).push().unwrap();
    table
}
pub fn create_email_pdf(emails: Vec<String>, contact_list_name: &str, region_name: &str) {
    let mut table = create_table_with_one_column("Emails");
    push_table_data_emails(emails, &mut table);
    let mut document = build_document();
    document_configuration(&mut document, "Email List", "Emails in the Specified List");
    document.push(Break::new(1.0));
    document.push(
        Paragraph::new(format!("Contact List Name: {}", contact_list_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(
        Paragraph::new(format!("Region Name: {}", region_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(Break::new(1.0));
    document.push(table);
    match document.render_to_file("Emails.pdf") {
        Ok(_) => println!(
            "The '{}' is also generated with the name {} in the current directory\n",
            "PDF".green().bold(),
            "'Emails.pdf'".green().bold()
        ),
        Err(_) => println!(
            "{}\n",
            "Error while generating Email 'PDF'".bright_red().bold()
        ),
    }
}
fn push_table_data_emails(emails: Vec<String>, table: &mut TableLayout) {
    for email in emails.iter() {
        table
            .row()
            .element(
                Paragraph::new(format!("{}", email))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(34, 91, 247)).bold()),
            )
            .push()
            .unwrap();
    }
}
pub fn create_text_result_pdf(
    headers: &Vec<&str>,
    records: Vec<String>,
    job_id: String,
    (bucket_name, video_name): (String, String),
) {
    let mut table = create_table("Text Information", "Predictions");
    push_table_data_text_results(headers, records, &mut table);
    let mut document = build_document();
    document_configuration(
        &mut document,
        "Text Detection Results",
        "Result of Start Text Detection Task",
    );
    document.push(Break::new(1.0));
    document.push(
        Paragraph::new(format!("Job ID:  {}", job_id))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(
        Paragraph::new(format!("Bucket Name:  {}", bucket_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(
        Paragraph::new(format!("Key Text Video Name:  {}", video_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(Break::new(1.0));
    document.push(table);
    match document.render_to_file("Text_Detection_Results.pdf") {
        Ok(_) => println!(
            "The '{}' is also generated with the name {} in the current directory\n",
            "PDF".green().bold(),
            "'Text_Detection_Results.pdf'".green().bold()
        ),
        Err(_) => println!(
            "{}\n",
            "Error while generating Text Detection Results 'PDF'"
                .bright_red()
                .bold()
        ),
    }
}
pub fn create_detect_face_image_pdf(bucket_name: &str, path_prefix: &str) {
    let mut document = build_document();
    document_configuration(&mut document, "DetectFaces", "Result of DetectFaces");
    document.push(Break::new(1.0));
    document.push(
        Paragraph::new(format!("Bucket Name:  {}", bucket_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(Break::new(1.0));
    document.push(
        Paragraph::new(format!("Bucket Path Prefix:  {}", path_prefix))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(Break::new(1.0));
    push_images_into_document(&mut document);
    match document.render_to_file("DetectFaces.pdf") {
        Ok(_) => println!(
            "The '{}' is also generated with the name {} in the current directory\n",
            "PDF".green().bold(),
            "'DetectFaces.pdf'".green().bold()
        ),
        Err(_) => println!(
            "{}\n",
            "Error while generating DetectFaces 'PDF'"
                .bright_red()
                .bold()
        ),
    }
}
fn push_images_into_document(document: &mut Document) {
    let face_image_dir = "face_details_images/";
    let entries = read_dir(face_image_dir).expect("No DIR is exist\n");
    for path in entries {
        let path = path.unwrap();
        match path.file_name().to_str() {
            Some(image_name) => {
                let image_path = format!("{}{}", face_image_dir, image_name);
                document.push(
                    Paragraph::new(format!("Image Name: {}", image_name))
                        .aligned(Alignment::Center)
                        .styled(Style::new().with_color(Color::Rgb(0, 128, 0))),
                );
                document.push(Break::new(1.0));
                document.push(
                    Image::from_path(image_path)
                        .expect("Unable to Load Image")
                        .with_alignment(Alignment::Center),
                );
                document.push(Break::new(2));
            }
            None => println!("Error while Walking the Directory\n"),
        }
    }
}
pub fn create_face_result_pdf(
    headers: &Vec<&str>,
    records: Vec<String>,
    job_id: &str,
    (bucket_name, video_name): (String, String),
) {
    let mut table = create_table("Face Information", "Predictions");
    push_table_data_face_results(headers, records, &mut table);
    let mut document = build_document();
    document_configuration(
        &mut document,
        "Face Detection Results",
        "Result of Start Face Detection Task",
    );
    document.push(Break::new(1.0));
    document.push(
        Paragraph::new(format!("Job ID:  {}", job_id))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(
        Paragraph::new(format!("Bucket Name:  {}", bucket_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(
        Paragraph::new(format!("Key Face Video Name:  {}", video_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(Break::new(1.0));
    document.push(table);
    match document.render_to_file("Face_Detection_Results.pdf") {
        Ok(_) => println!(
            "The '{}' is also generated with the name {} in the current directory\n",
            "PDF".green().bold(),
            "'Face_Detection_Results.pdf'".green().bold()
        ),
        Err(_) => println!(
            "{}\n",
            "Error while generating face Detection Results 'PDF'"
                .bright_red()
                .bold()
        ),
    }
}
fn push_table_data_text_results(
    headers: &Vec<&str>,
    records: Vec<String>,
    table: &mut TableLayout,
) {
    let mut count = 0;
    for (record, header) in records.into_iter().zip(headers.into_iter().cycle()) {
        table
            .row()
            .element(
                Paragraph::new(format!("{}", header))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(34, 91, 247)).bold()),
            )
            .element(
                Paragraph::new(format!("{}", record))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(208, 97, 0)).bold()),
            )
            .push()
            .unwrap();
        count += 1;
        if count % 4 == 0 {
            table
                .row()
                .element(Break::new(1.0))
                .element(Break::new(1.0))
                .push()
                .unwrap();
        }
    }
}
fn push_table_data_face_results(
    headers: &Vec<&str>,
    records: Vec<String>,
    table: &mut TableLayout,
) {
    let mut count = 0;
    for (record, header) in records.into_iter().zip(headers.into_iter().cycle()) {
        table
            .row()
            .element(
                Paragraph::new(format!("{}", header))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(34, 91, 247)).bold()),
            )
            .element(
                Paragraph::new(format!("{}", record))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(208, 97, 0)).bold()),
            )
            .push()
            .unwrap();
        count += 1;
        if count % 9 == 0 {
            table
                .row()
                .element(Break::new(1.0))
                .element(Break::new(1.0))
                .push()
                .unwrap();
        }
    }
}
pub fn create_email_identities_pdf(
    headers: &Vec<&str>,
    identities: Vec<String>,
    region_name: &str,
) {
    let mut table = create_table("Identity Info", "Values");
    push_table_data_emails_identies(headers, identities, &mut table);
    let mut document = build_document();
    document_configuration(
        &mut document,
        "Email Identies",
        "Result of List Email Identities",
    );
    document.push(Break::new(1.0));
    document.push(
        Paragraph::new(format!("Region Name:  {}", region_name))
            .aligned(Alignment::Left)
            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
    );
    document.push(Break::new(1.0));
    document.push(table);
    match document.render_to_file("EmailIdentitiesInfo.pdf") {
        Ok(_) => println!(
            "The '{}' is also generated with the name {} in the current directory\n",
            "PDF".green().bold(),
            "'EmailIdentitiesInfo.pdf'".green().bold()
        ),
        Err(_) => println!(
            "{}\n",
            "Error while generating Text Detection Results 'PDF'"
                .bright_red()
                .bold()
        ),
    }
}
pub fn push_table_data_emails_identies(
    headers: &Vec<&str>,
    identities: Vec<String>,
    table: &mut TableLayout,
) {
    let mut count = 0;
    for (record, header) in identities.into_iter().zip(headers.into_iter().cycle()) {
        table
            .row()
            .element(
                Paragraph::new(format!("{}", header))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(34, 91, 247)).bold()),
            )
            .element(
                Paragraph::new(format!("{}", record))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(208, 97, 0)).bold()),
            )
            .push()
            .unwrap();
        count += 1;
        if count % 4 == 0 {
            table
                .row()
                .element(Break::new(1.0))
                .element(Break::new(1.0))
                .push()
                .unwrap();
        }
    }
}
