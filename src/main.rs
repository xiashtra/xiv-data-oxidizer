use std::path::Path;
use std::{env, error::Error};

use ironworks::{
    Ironworks,
    excel::{Excel, Language},
    sqpack::{Install, SqPack},
};
mod exd_schema;
mod export;
mod formatter;

fn available_languages(ironworks: &Ironworks) -> Vec<Language> {
    // Wokaround used by boilmaster, ironworks currently doesn't support directly checking if a file exists.
    struct FileExists;
    impl ironworks::file::File for FileExists {
        fn read(_stream: impl ironworks::FileStream) -> Result<Self, ironworks::Error> {
            Ok(Self)
        }
    }

    ironworks
        .file::<ironworks::file::exh::ExcelHeader>("exd/Item.exh")
        .expect("Could not read 'exd/Item.exh'")
        .languages
        .into_iter()
        .map(Language::from)
        .filter(|language| {
            // Check if the files actually exist. The Global version's `EXcelHeader`s indicate that
            // all languages are supported, even though the files aren't present besides for EN, DE, FR, and JA.
            ironworks
                .file::<FileExists>(&format!(
                    "exd/Item_0_{}.exd",
                    export::language_code(language)
                ))
                .is_ok()
        })
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        panic!(
            "You must provide a game path. For example: cargo run -- \"C:\\Program Files (x86)\\Square Enix\\FINAL FANTASY XIV - A Realm Reborn\""
        );
    }

    let path = Path::new(&args[1]);

    let ironworks = Ironworks::new().with_resource(SqPack::new(Install::at(path)));
    let languages = available_languages(&ironworks);
    let mut excel = Excel::new(ironworks);

    for language in languages {
        excel.set_default_language(language);
        let sheets = excel.list().expect("Could not retrieve sheet list.");

        println!(
            "Exporting {} sheets",
            export::language_code(&language).to_uppercase()
        );

        for sheet in sheets.iter() {
            match export::sheet(&excel, language, &sheet) {
                Ok(_) => (),
                // Log failed sheets and continue
                Err(err) => eprintln!("Failed to export {}. {}", sheet, err),
            }
        }
    }

    // Quick debugging for schema updates

    // for language in LANGUAGES {
    //     excel.set_default_language(language);
    //     export::sheet(&excel, language, &String::from("Mount"))?;
    // }

    // let language = Language::English;
    // excel.set_default_language(language);
    // export::sheet(&excel, language, "Mount")?;

    Ok(())
}
