use colored::Colorize;
use dotenv::dotenv;
use genpdf::{
    elements::{Break, FrameCellDecorator, Paragraph, TableLayout},
    fonts,
    style::{Color, Style},
    Alignment, Document, Element, PaperSize, SimplePageDecorator,
};
fn build_document() -> Result<Document, &'static str> {
    let builtin_font = Some(fonts::Builtin::Times);
    dotenv().ok();
    match std::env::var("FONT_PATH") {
        Ok(path) => {
            match std::env::var("FONT_NAME") {
                    Ok(font_name) => {
                        // let default_font_family = fonts::from_files(&path,&font_name, builtin_font).expect("An error has occurred while attempting to read from the specified fonts directory.\nYou can find additional information at this link: 'https://tinyurl.com/4j8dyhap'\n\n");
                        let default_font_family =
                fonts::from_files(&path, &font_name, builtin_font).expect("An error has occurred while attempting to read from the specified fonts directory.\nYou can find additional information at this link: 'https://tinyurl.com/4j8dyhap'\n\n");
                        Ok(Document::new(default_font_family))
                    }
                    Err(_) => Err("The PDF generation process cannot proceed because the essential environment variable,'FONT_NAME', has not been set. Instead, a text document is generated"),
                }
        }
        Err(_) => {
            let error_msg ="The PDF generation process cannot proceed because the essential environment variable, 'FONT_PATH', and 'FONT_NAME' has not been set. Instead, a text document is generated";
            Err(error_msg)
        }
    }
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
pub fn create_text_result_pdf(
    headers: &Vec<&str>,
    records: Vec<String>,
    job_id: String,
    (bucket_name, video_name): (String, String),
) {
    let mut table = create_table("Text Information", "Predictions");
    push_table_data_text_results(headers, records, &mut table);
    match build_document() {
        Ok(mut document) => {
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
        Err(err) => println!("{err}"),
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
    match build_document() {
        Ok(mut document) => {
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
                    "Error while generating Text Detection Results 'PDF'"
                        .bright_red()
                        .bold()
                ),
            }
        }
        Err(err) => println!("{err}"),
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
