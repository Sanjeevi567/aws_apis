face_detail.into_iter().for_each(|face_details| {
    let timestamp = face_details.timestamp;
    let format_timestamp = format!("Timestamp: {timestamp}\n");
    file.write_all(format_timestamp.as_bytes()).unwrap();
    if let Some(face_detail_type) = face_details.face {
        let mut wrap_face_detection = FaceDetails::build(face_detail_type);
        let gender = wrap_face_detection.gender();
        let mut gender_string = String::new();
        if let (Some(gender_), Some(conf_level)) = (gender.0, gender.1) {
            gender_string.push_str(&gender_);
            gender_string.push_str(" and ");
            let confidence_level = format!("{}", conf_level);
            gender_string.push_str(&confidence_level);
        }
        let buf = format!("Gender and Confidence Level: {}\n",gender_string);
        file.write_all(buf.as_bytes()).unwrap();

        let age_range = wrap_face_detection.age_range();
        let mut age_range_string = String::new();
        if let (Some(low),Some(high)) = (age_range.0,age_range.1) {
            let format_age_range = format!("The lowest age prediction is {low}, and the highest age prediction is {high}");
            age_range_string.push_str(&format_age_range);
        }
        let buf = format!("Age Range in Years: {}\n",age_range_string);
        file.write_all(buf.as_bytes()).unwrap();

        let smile = wrap_face_detection.smile();
        let mut smile_string = String::new();
        if let (Some(smiling),Some(conf_level)) = (smile.0,smile.1) {
            let format_smile = format!("{smiling},{conf_level}");
            smile_string.push_str(&format_smile);
        }
        let buf = format!("Is the Face Smiling and Confidence Level: {}\n",smile_string);
        file.write_all(buf.as_bytes()).unwrap();

        let beard = wrap_face_detection.beard();
        let mut beard_string = String::new();
        if let (Some(beard),Some(conf_level)) =(beard.0,beard.1)  {
            let format_beard = format!("{beard},{conf_level}");
            beard_string.push_str(&format_beard);
        }
        let buf = format!("Has a Beard and Confidence Level: {}\n",beard_string);
        file.write_all(buf.as_bytes()).unwrap();

        let mustache = wrap_face_detection.mustache();
        let mut mustache_string = String::new();
        if let (Some(mustache),Some(conf_level)) =(mustache.0,mustache.1)  {
            let format_mustache = format!("{mustache},{conf_level}");
            mustache_string.push_str(&format_mustache);
        }
        let buf = format!("Has a Mustache and Confidence Level: {}\n",mustache_string);
        file.write_all(buf.as_bytes()).unwrap();

        let sunglasses = wrap_face_detection.sunglasses();
        let mut sunglasses_string = String::new();
        if let (Some(sun),Some(conf_level)) = (sunglasses.0,sunglasses.1) {
            let format_sunglass = format!("{sun},{conf_level}");
            sunglasses_string.push_str(&format_sunglass);
        }
        let buf = format!("Has Sunglasses and Confidence Level: {}\n",sunglasses_string);
        file.write_all(buf.as_bytes()).unwrap();

        let eyeglasses = wrap_face_detection.eyeglasses();
        let mut eyeglasses_string = String::new();
        if let (Some(eye),Some(conf_level)) =(eyeglasses.0,eyeglasses.1)  {
            let format_eyeglasses = format!("{eye},{conf_level}");
            eyeglasses_string.push_str(&format_eyeglasses);
        }
        let buf = format!("Has Eyeglasses and Confidence Level: {}\n",eyeglasses_string);
        file.write_all(buf.as_bytes()).unwrap();

        let bounding_box = wrap_face_detection.bounding_box();
        let mut bounding_string = String::new();
        if let (Some(width),Some(height),Some(left),Some(top)) = (bounding_box.0,bounding_box.1,bounding_box.2,bounding_box.3) {
            let format_bounding_box = format!("Width: {width},Height: {height},Left: {left},Top: {top}");
            bounding_string.push_str(&format_bounding_box);
        }
        let buf = format!("Bounding Box Details: {}\n\n\n",bounding_string);
        file.write_all(buf.as_bytes()).unwrap();

        face_details_vector.push(timestamp.to_string());
        face_details_vector.push(gender_string);
        face_details_vector.push(age_range_string);
        face_details_vector.push(smile_string);
        face_details_vector.push(beard_string);
        face_details_vector.push(mustache_string);
        face_details_vector.push(sunglasses_string);
        face_details_vector.push(eyeglasses_string);
        face_details_vector.push(bounding_string);
        
    }
});