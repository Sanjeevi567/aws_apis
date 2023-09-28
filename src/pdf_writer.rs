use colored::Colorize;
use genpdf::{
    elements::{Break, FrameCellDecorator, Image, Paragraph, TableLayout},
    fonts::{FontData, FontFamily},
    style::{Color, Style},
    Alignment, Document, Element, PaperSize, SimplePageDecorator,
};
use printpdf;
use printpdf::types::plugins::graphics::two_dimensional::font::BuiltinFont;
use regex::Regex;
use std::{
    fs::{create_dir, read_dir, remove_dir_all, OpenOptions},
    io::Write,
};
pub fn build_document() -> Document {
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
pub fn document_configuration(document: &mut Document, title: &str, page_title: &str) {
    document.set_title(title);
    document.set_minimal_conformance();
    document.set_line_spacing(1.50);
    document.push(
        Paragraph::new(page_title)
            .aligned(Alignment::Center)
            .styled(Style::new().bold()),
    );
    let mut page_decorator = SimplePageDecorator::default();
    page_decorator.set_margins(20);
    document.set_page_decorator(page_decorator);
    document.set_paper_size(PaperSize::A4);
}
pub fn create_table(key: &str, value: &str) -> TableLayout {
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
//create tempdir/ before calling this function.
pub async fn create_celebrity_pdf(
    face_info: Vec<String>,
    predictions: Vec<String>,
    local_image_path: Option<&str>,
    (bucket_name, key_image_name): (Option<&str>, Option<&str>),
) {
    let mut table = create_table("Celebrity Information", "Predictions");
    push_table_data_celebrity_results(face_info, predictions, &mut table);
    let mut document = build_document();
    document_configuration(
        &mut document,
        "Celebrity",
        "Result of Recognize Celebrities",
    );
    document.push(Break::new(1.0));
    match local_image_path {
        Some(image_path) => {
            document.push(
                Paragraph::new(format!("Local Image Path:  {}", image_path))
                    .aligned(Alignment::Center)
                    .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
            );
        }
        None => match (bucket_name, key_image_name) {
            (Some(bucket_name), Some(key_image_name)) => {
                document.push(
                    Paragraph::new(format!("Bucket Name:  {}", bucket_name))
                        .aligned(Alignment::Center)
                        .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
                );
                document.push(
                    Paragraph::new(format!("Key Image Name:  {}", key_image_name))
                        .aligned(Alignment::Center)
                        .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
                );
            }
            _ => println!("This shouldn't have happened\n"),
        },
    }

    document.push(Break::new(1.0));
    match local_image_path {
        Some(local_image) => {
            let image = image::open(&local_image)
                .expect("Error while opening the image\n")
                .resize_to_fill(800, 800, image::imageops::FilterType::Gaussian);
            create_dir("tempdir/").expect("Error while creating tempdir for resizing the image");
            image
                .save("tempdir/image.png")
                .expect("Error while writing Image to tempdir\n");
            document.push(
                Image::from_path("tempdir/image.png")
                    .expect("Error reading the image path\n")
                    .with_alignment(Alignment::Center),
            );
            document.push(table);
            let file_name_pattern =
                Regex::new(r#"[^\\/:\*\?"<>\|]+$"#).expect("Error while parsing Regex Syntax\n");
            let file_name = file_name_pattern
                .find_iter(local_image)
                .map(|as_str| as_str.as_str())
                .collect::<Vec<&str>>();
            let mut file_name = file_name.join("");
            if file_name.starts_with("/") {
                file_name.remove(0);
            };
            let pdf_name = format!("CelebrityResults_{}.pdf", file_name);
            match document.render_to_file(&pdf_name) {
                Ok(_) => println!(
                    "The '{}' is generated with the name {} in the current directory\n",
                    "PDF".green().bold(),
                    pdf_name.green().bold()
                ),
                Err(_) => {
                    println!(
                        "{}",
                        "The file name below is causing problems during the PDF generation process"
                            .yellow()
                            .bold()
                    );
                    println!("{}\n", file_name.blue().bold());
                    println!(
                        "{}\n",
                        "Error while generating Celebrity Results 'PDF'"
                            .bright_red()
                            .bold()
                    );
                }
            }
            remove_dir_all("tempdir/").expect("Error while deleting tempdir/");
        }
        None => {
            if let (Some(bucket_name), Some(key_image_name)) = (bucket_name, key_image_name) {
                let sdk_config = aws_config::load_from_env().await;
                let client = aws_sdk_s3::Client::new(&sdk_config);
                let output = client
                    .get_object()
                    .bucket(bucket_name)
                    .key(key_image_name)
                    .send()
                    .await
                    .expect("Error while Getting Object for specified Bucket and Key Name\n");
                let have_slash_and_dot_pattern =
                    Regex::new(r#"([^./]+)\.([^/]+)"#).expect("Error while parsing Regex Syntax\n");
                let have_slash_but_no_extension_pattern =
                    Regex::new(r#"/([^/]+)$"#).expect("Error while parsing regex syntax");
                let no_slash_no_dot_pattern =
                    Regex::new(r#"^[^./]*$"#).expect("Error while parsing Regex syntax");
                let file_name: Vec<String> = if have_slash_and_dot_pattern.is_match(key_image_name)
                {
                    have_slash_and_dot_pattern
                        .find_iter(key_image_name)
                        .map(|string| string.as_str().to_string())
                        .collect()
                } else if have_slash_but_no_extension_pattern.is_match(key_image_name) {
                    have_slash_but_no_extension_pattern
                        .find_iter(key_image_name)
                        .map(|string| string.as_str().to_string())
                        .collect()
                } else {
                    no_slash_no_dot_pattern
                        .find_iter(key_image_name)
                        .map(|string| string.as_str().to_string())
                        .collect()
                };
                let mut file_name = file_name.join("");
                if file_name.starts_with("/") {
                    file_name.remove(0);
                };
                let mut file = OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(&file_name)
                    .expect("Error while creating file\n");
                let bytes = output
                    .body
                    .collect()
                    .await
                    .expect("Error While Getting Bytes\n")
                    .into_bytes();
                file.write_all(&*bytes)
                    .expect("Error while writing bytes\n");
                let image = image::open(&file_name)
                    .expect("Error while opening the image\n")
                    .resize_to_fill(800, 800, image::imageops::FilterType::Gaussian);
                create_dir("tempdir/")
                    .expect("Error while creating tempdir for resizing the image");
                image
                    .save("tempdir/image.png")
                    .expect("Error while writing Image to tempdir\n");
                document.push(
                    Image::from_path("tempdir/image.png")
                        .expect("Error reading the image path\n")
                        .with_alignment(Alignment::Center),
                );
                remove_dir_all("tempdir/").expect("Error while deleting tempdir/");
                document.push(table);
                let pdf_name = format!("CelebrityResults_{}.pdf", file_name);
                match document.render_to_file(&pdf_name) {
                    Ok(_) => println!(
                        "The '{}' is generated with the name {} in the current directory\n",
                        "PDF".green().bold(),
                        pdf_name.green().bold()
                    ),
                    Err(_) => {
                        println!(
                        "{}",
                        "The file name below is causing problems during the PDF generation process"
                            .yellow()
                            .bold()
                    );
                        println!("{}\n", file_name.blue().bold());
                        println!(
                            "{}\n",
                            "Error while generating Celebrity Results 'PDF'"
                                .bright_red()
                                .bold()
                        );
                    }
                }
            }
        }
    }
}
pub async fn create_celebrity_single_pdf(
    local_image_dir: Option<&str>,
    entries: Option<Vec<String>>,
    bucket_name: Option<&str>,
) {
    let mut document = build_document();
    document_configuration(
        &mut document,
        "Celebrity",
        "Result of Recognize Celebrities",
    );
    document.push(Break::new(1.0));

    match local_image_dir {
        Some(directory) => {
            let entries = read_dir(directory).expect("Error finding the directory\n");
            create_dir("tempdir/").expect("Error while creating tempdir/ for resizing images\n");
            for entry in entries {
                let local_image_path = entry.unwrap().file_name();
                match local_image_path.to_str() {
                    Some(image_path) => {
                        let full_image_path = format!("{directory}/{image_path}");
                        document.push(
                            Paragraph::new(format!("Local Image Path:  {}", full_image_path))
                                .aligned(Alignment::Center)
                                .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
                        );
                        let image = image::open(&full_image_path)
                            .expect("Error while opening the image\n")
                            .resize_to_fill(800, 600, image::imageops::FilterType::Gaussian);
                        image
                            .save("tempdir/image.png")
                            .expect("Error while writing Image to tempdir\n");
                        document.push(Break::new(1.0));
                        document.push(
                            Image::from_path("tempdir/image.png")
                                .expect("Error reading the image path\n")
                                .with_alignment(Alignment::Center),
                        );

                        let table_data =
                            recognize_celebrities(Some(&full_image_path), None, None, 0).await;
                        let table = create_table("Celebrity Information", "Predictions");
                        push_each_table_data_celebrity_results_to_a_document(
                            table_data.0,
                            table_data.1,
                            table,
                            &mut document,
                        );
                    }
                    None => println!("{}\n", "No Image File Name is Found".red().bold()),
                }
            }
            remove_dir_all("tempdir/").expect("Error while deleting tempdir/");
        }
        None => match bucket_name {
            Some(bucket_name) => {
                let have_slash_and_dot_pattern =
                    Regex::new(r#"([^./]+)\.([^/]+)"#).expect("Error while parsing Regex Syntax\n");
                let have_slash_but_no_extension_pattern =
                    Regex::new(r#"/([^/]+)$"#).expect("Error while parsing regex syntax");
                let no_slash_no_dot_pattern =
                    Regex::new(r#"^[^./]*$"#).expect("Error while parsing Regex syntax");
                create_dir("tempdir/")
                    .expect("Error while creating tempdir/ for resizing images\n");
                create_dir("DownloadedImages/")
                    .expect("Error while creating DownloadedImages/ for writing images from S3 bucket\n");
                let mut count = 0;
                for key_image_name in entries.unwrap().into_iter().skip(1) {
                    let sdk_config = aws_config::load_from_env().await;
                    let client = aws_sdk_s3::Client::new(&sdk_config);
                    let outputs = client
                        .get_object()
                        .bucket(bucket_name)
                        .key(&key_image_name)
                        .send()
                        .await
                        .expect("Error while Downloading Content from Given Prefix or Bucket\n");
                    let file_name: Vec<String> =
                        if have_slash_and_dot_pattern.is_match(&key_image_name) {
                            have_slash_and_dot_pattern
                                .find_iter(&key_image_name)
                                .map(|string| string.as_str().to_string())
                                .collect()
                        } else if have_slash_but_no_extension_pattern.is_match(&key_image_name) {
                            have_slash_but_no_extension_pattern
                                .find_iter(&key_image_name)
                                .map(|string| string.as_str().to_string())
                                .collect()
                        } else {
                            no_slash_no_dot_pattern
                                .find_iter(&key_image_name)
                                .map(|string| string.as_str().to_string())
                                .collect()
                        };
                    let mut file_name = file_name.join("");
                    if file_name.starts_with("/") {
                        file_name.remove(0);
                    };
                    let file_path = format!("DownloadedImages/{file_name}");
                    let bytes = outputs.body.collect().await.unwrap();
                    let bytes = bytes.into_bytes();
                    let mut file = OpenOptions::new()
                        .create(true)
                        .read(true)
                        .write(true)
                        .open(&file_path)
                        .expect("Error while creating file\n");
                    file.write_all(&*bytes).unwrap();
                    document.push(
                        Paragraph::new(format!("Bucket Name:  {}", bucket_name))
                            .aligned(Alignment::Center)
                            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
                    );
                    document.push(
                        Paragraph::new(format!("Key Image Name:  {}", key_image_name))
                            .aligned(Alignment::Center)
                            .styled(Style::new().with_color(Color::Rgb(0, 128, 0)).bold()),
                    );
                    document.push(Break::new(1.0));
                    let image = image::open(&file_path)
                        .expect("Error while opening the image\n")
                        .resize_to_fill(800, 600, image::imageops::FilterType::Gaussian);
                    image
                        .save("tempdir/image.png")
                        .expect("Error while writing Image to tempdir\n");
                    document.push(
                        Image::from_path("tempdir/image.png")
                            .expect("Error reading the image path\n")
                            .with_alignment(Alignment::Center),
                    );

                    let table_data = recognize_celebrities(
                        None,
                        Some(&bucket_name),
                        Some(&key_image_name),
                        count,
                    )
                    .await;
                    count += 1;
                    let table = create_table("Celebrity Information", "Predictions");
                    push_each_table_data_celebrity_results_to_a_document(
                        table_data.0,
                        table_data.1,
                        table,
                        &mut document,
                    );
                }
                remove_dir_all("tempdir/").expect("Error while deleting tempdir/");
            }
            _ => println!("This shouldn't have happened\n"),
        },
    }

    match document.render_to_file("CelebrityResults.pdf") {
        Ok(_) => println!(
            "The '{}' is generated with the name '{}' in the current directory\n",
            "PDF".green().bold(),
            "CelebrityResults.pdf".green().bold(),
        ),
        Err(_) => {
            println!(
                "{}\n",
                "Error while generating Celebrity Results 'PDF'"
                    .bright_red()
                    .bold()
            );
        }
    }
}
async fn recognize_celebrities(
    local_image_path: Option<&str>,
    bucket_name: Option<&str>,
    image_key_name: Option<&str>,
    count: usize,
) -> (Vec<String>, Vec<String>) {
    use aws_sdk_pinpoint::primitives::Blob;
    use aws_sdk_rekognition::types::{Image as S3Image, S3Object};
    use std::io::Read;
    let config = aws_config::load_from_env().await;
    let client = aws_sdk_rekognition::Client::new(&config);
    let image = match local_image_path {
        Some(local_image_path) => {
            let mut file = std::fs::File::open(local_image_path)
                .expect("Error while reading the path you specified\n");
            let mut vec_of_u8s = Vec::new();
            file.read_to_end(&mut vec_of_u8s).unwrap();
            let bytes_builder = Blob::new(vec_of_u8s);
            S3Image::builder().bytes(bytes_builder).build()
        }
        None => {
            let s3_object_builder = S3Object::builder()
                .set_bucket(bucket_name.map(|to_str| to_str.to_string()))
                .set_name(image_key_name.map(|to_str| to_str.to_string()))
                .build();
            S3Image::builder().s3_object(s3_object_builder).build()
        }
    };

    let outputs = client
        .recognize_celebrities()
        .image(image)
        .send()
        .await
        .expect("Error while Recognizing Celebrities\n");
    let headers = vec![
        "Celebrity Name".into(),
        "Unique Celebrity Identifier".into(),
        "Gender of the Celebrity".into(),
        "Face Location of the Celebrity's Face".into(),
        "Is the Celebrity Smiling?".into(),
    ];
    let mut records = Vec::new();
    let file_path = format!("CelebrityDetails_{count}.txt");
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&file_path)
        .expect("Error while creating file\n");
    if let Some(celebrity_faces) = outputs.celebrity_faces {
        celebrity_faces.into_iter().for_each(|faces| {
            if let Some(name) = faces.name {
                let famous_for = match name.as_str() {
                    "Ralph Fiennes" => format!("{name}(voldemort)"),
                    "Chris Hemsworth" => format!("{name}(Odin Son Thor)"),
                    "Tom Hiddleston" => format!("{name}(Loki)"),
                    "Johnny Depp" => format!("{name}(Captain Jack Sparrow)"),
                    "Tom Holland" => format!("{name}(Little Spider Man)"),
                    "Tobey Maguire" => format!("{name}(Amazing Spider Man)"),
                    "Andrew Garfield" => format!("{name}(Amazing Spider Man)"),
                    "Robert Downey Jr." => format!("{name}(Iron Man)"),
                    "Tom Felton" => format!("{name}(Draco Malfoy)"),
                    "Rupert Grint" => format!("{name}(Ron Weasley)"),
                    "Emma Watson" => format!("{name}(Hermione Granger)"),
                    "Daniel Radcliffe" => format!("{name}(Harry Potter)"),
                    "Mark Alan" => format!("{name}(Hulk)"),
                    "Chris Evans" => format!("{name}(Captain America)"),
                    "Chadwick Boseman" => format!("{name}(Black Panther)"),
                    "Vin Diesel" => format!("{name}(Dominic Toretto,I'm Groot)"),
                    "Paul Walker" => format!("{name}(Car Racer)"),
                    "Joseph Vijay" => format!("{name}(Ilaya Thalapathy)"),
                    "Rajinikanth" => format!("{name}(Super Star)"),
                    "Kamal Haasan" => format!("{name}(Ulaga Nayagan)"),
                    "Ajith Kumar" => format!("{name}(Ultimate Star)"),
                    _ => format!("{name}"),
                };
                println!("Celebrity Name: {}", famous_for.green().bold());
                let buf = format!("Celebrity Name: {}\n", famous_for);
                file.write_all(buf.as_bytes()).unwrap();
                records.push(famous_for);
            }
            if let Some(id) = faces.id {
                println!("Celebrity Amazon ID: {}", id.green().bold());
                let buf = format!("Celebrity Amazon ID: {}\n", id);
                file.write_all(buf.as_bytes()).unwrap();
                records.push(id);
            }
            if let Some(gender) = faces.known_gender {
                let gender_ = gender.r#type;
                if let Some(genderr) = gender_ {
                    let finall = genderr.as_str().to_string();
                    println!("Celebrity Gender: {}", finall.green().bold());
                    let buf = format!("Celebrity Gender: {}\n", finall);
                    file.write_all(buf.as_bytes()).unwrap();
                    records.push(finall);
                }
            }
            if let Some(face) = faces.face {
                let mut bbox_string = String::new();
                if let Some(bbox) = face.bounding_box {
                    if let (Some(width), Some(height), Some(left), Some(top)) =
                        (bbox.width, bbox.height, bbox.left, bbox.top)
                    {
                        let format_bbox = format!(
                            "Width: {width:.2},Height: {height:.2},Left: {left:.2},Top: {top:.2}"
                        );
                        println!(
                            "Celebrity Bounding Box Details: {}",
                            format_bbox.green().bold()
                        );
                        let buf = format!("Celebrity Bounding Box Details: {}\n", format_bbox);
                        file.write_all(buf.as_bytes()).unwrap();
                        bbox_string.push_str(&format_bbox);
                    }
                }
                records.push(bbox_string);
                if let Some(smile) = face.smile {
                    let format_smile = format!("{}", smile.value);
                    println!("Is Celebrity Smiling?: {}\n", format_smile.green().bold());
                    let buf = format!("Is Celebrity Smiling?: {}\n", format_smile);
                    file.write_all(buf.as_bytes()).unwrap();
                    records.push(format_smile);
                }
            }
        });
    }
    match std::fs::File::open(file_path) {
        Ok(_) => println!(
            "{}\n",
            "The text file has been successfully written to the current directory"
                .green()
                .bold()
        ),
        Err(_) => println!("{}\n", "Error while writing File".red().bold()),
    }
    (headers, records)
}
pub fn push_each_table_data_celebrity_results_to_a_document(
    headers: Vec<String>,
    records: Vec<String>,
    mut table: TableLayout,
    document: &mut Document,
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
        if count % 5 == 0 {
            table
                .row()
                .element(Break::new(1.0))
                .element(Break::new(1.0))
                .push()
                .unwrap();
        }
    }
    document.push(table);
    document.push(Break::new(1.0));
}
pub fn push_table_data_celebrity_results(
    headers: Vec<String>,
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
        if count % 5 == 0 {
            table
                .row()
                .element(Break::new(1.0))
                .element(Break::new(1.0))
                .push()
                .unwrap();
        }
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
