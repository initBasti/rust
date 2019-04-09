/// This module contains source for the `generate` command.
use exercise::{self, get, val_as, Result};
use failure::format_err;
use serde_json::Value as JsonValue;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

const GITIGNORE_CONTENT: &str = include_str!("defaults/gitignore");
const EXAMPLE_RS_CONTENT: &str = include_str!("defaults/example.rs");
const DESCRIPTION_MD_CONTENT: &str = include_str!("defaults/description.md");
const METADATA_YML_CONTENT: &str = include_str!("defaults/metadata.yml");

// Generate .meta directory and its contents without using the canonical data
fn generate_meta(exercise_name: &str, exercise_path: &Path) -> Result<()> {
    let meta_dir = exercise_path.join(".meta");
    fs::create_dir(&meta_dir)?;

    for (file, content) in [
        ("description.md", DESCRIPTION_MD_CONTENT),
        ("metadata.yml", METADATA_YML_CONTENT),
    ]
    .iter()
    {
        if !exercise::canonical_file_exists(exercise_name, file)? {
            fs::write(exercise_path.join(".meta").join(file), content)?;
        }
    }

    if fs::read_dir(&meta_dir)?.count() == 0 {
        fs::remove_dir(meta_dir)?;
    }

    Ok(())
}

fn parse_case(
    case: &JsonValue,
    property_functions: &mut HashMap<String, String>,
    test_functions: &mut Vec<String>,
    use_maplit: bool,
) -> Result<()> {
    if let Some(sub_cases) = case.get("cases") {
        for sub_case in val_as!(sub_cases, as_array) {
            parse_case(&sub_case, property_functions, test_functions, use_maplit)?;
        }
    }

    if let Some(property) = case.get("property") {
        let property = val_as!(property, as_str);

        if !property_functions.contains_key(property) {
            property_functions.insert(
                property.to_string(),
                exercise::generate_property_body(property),
            );
        }

        test_functions.push(exercise::generate_test_function(case, use_maplit)?);
    }

    Ok(())
}

// Generate test suite using the canonical data
fn generate_tests_from_canonical_data(
    exercise_name: &str,
    exercise_path: &Path,
    canonical_data: &JsonValue,
    use_maplit: bool,
) -> Result<()> {
    exercise::update_cargo_toml_version(exercise_name, canonical_data)?;

    let tests_path = exercise_path
        .join("tests")
        .join(format!("{}.rs", exercise_name));

    let tests_content = exercise::get_tests_content(exercise_name)?;

    let updated_tests_content = format!(
        "//! Tests for {exercise_name} \n\
        //! \n\
        //! Generated by [utility][utility] using [canonical data][canonical_data]\n\
        //! \n\
        //! [utility]: https://github.com/exercism/rust/tree/master/util/exercise\n\
        //! [canonical_data]: https://raw.githubusercontent.com/exercism/problem-specifications/master/exercises/{exercise_name}/canonical-data.json\n\
        \n\
        {} \n\
        ",
        tests_content,
        exercise_name=exercise_name,
    );

    fs::write(&tests_path, updated_tests_content)?;

    let mut property_functions: HashMap<String, String> = HashMap::new();

    let mut test_functions: Vec<String> = Vec::new();

    for case in get!(canonical_data, "cases", as_array) {
        parse_case(
            &case,
            &mut property_functions,
            &mut test_functions,
            use_maplit,
        )?;
    }

    if !test_functions.is_empty() {
        let first_test_function = test_functions.remove(0).replace("#[ignore]\n", "");

        test_functions.insert(0, first_test_function);
    }

    let mut tests_file = OpenOptions::new().append(true).open(&tests_path)?;

    for property_body in property_functions.values() {
        tests_file.write_all(property_body.as_bytes())?;
    }

    tests_file.write_all(test_functions.join("\n\n").as_bytes())?;

    exercise::rustfmt(&tests_path)?;

    Ok(())
}

// Run bin/configlet generate command to generate README for the exercise
fn generate_readme(exercise_name: &str) -> Result<()> {
    println!(
        "Generating README for {} via 'bin/configlet generate'",
        exercise_name
    );

    let problem_specifications_path = Path::new(&*exercise::TRACK_ROOT)
        .join("..")
        .join("problem-specifications");

    if !problem_specifications_path.exists() {
        let problem_specifications_url = "https://github.com/exercism/problem-specifications.git";
        println!(
            "problem-specifications repository not found. Cloning the repository from {}",
            problem_specifications_url
        );

        Command::new("git")
            .current_dir(&*exercise::TRACK_ROOT)
            .stdout(Stdio::inherit())
            .arg("clone")
            .arg(problem_specifications_url)
            .arg(&problem_specifications_path)
            .output()?;
    }

    exercise::run_configlet_command(
        "generate",
        &[
            ".",
            "--only",
            exercise_name,
            "--spec-path",
            problem_specifications_path
                .to_str()
                .ok_or(format_err!("path inexpressable as str"))?,
        ],
    )?;

    Ok(())
}

// Generate a new exercise with specified name and flags
pub fn generate_exercise(exercise_name: &str, use_maplit: bool) -> Result<()> {
    if exercise::exercise_exists(exercise_name) {
        return Err(format_err!("exercise with the name {} already exists", exercise_name,).into());
    }

    let exercise_path = Path::new(&*exercise::TRACK_ROOT)
        .join("exercises")
        .join(exercise_name);

    println!(
        "Generating a new exercise at path: {}",
        exercise_path
            .to_str()
            .ok_or(format_err!("path inexpressable as str"))?
    );

    let _cargo_new_output = Command::new("cargo")
        .arg("new")
        .arg("--lib")
        .arg(
            exercise_path
                .to_str()
                .ok_or(format_err!("path inexpressable as str"))?,
        )
        .output()?;

    fs::write(exercise_path.join(".gitignore"), GITIGNORE_CONTENT)?;

    if use_maplit {
        let mut cargo_toml_file = OpenOptions::new()
            .append(true)
            .open(exercise_path.join("Cargo.toml"))?;

        cargo_toml_file.write_all(b"maplit = \"1.0.1\"")?;
    }

    fs::create_dir(exercise_path.join("tests"))?;

    let mut test_file = File::create(
        exercise_path
            .join("tests")
            .join(format!("{}.rs", exercise_name)),
    )?;

    if use_maplit {
        test_file.write_all(b"#[macro_use]\nextern crate maplit;\n")?;
    }

    test_file
        .write_all(&format!("extern crate {};\n", exercise_name.replace("-", "_")).into_bytes())?;

    test_file
        .write_all(&format!("use {}::*;\n\n", exercise_name.replace("-", "_")).into_bytes())?;

    fs::write(exercise_path.join("example.rs"), EXAMPLE_RS_CONTENT)?;

    match exercise::get_canonical_data(exercise_name) {
        Ok(canonical_data) => {
            println!("Generating tests from canonical data");

            generate_tests_from_canonical_data(
                &exercise_name,
                &exercise_path,
                &canonical_data,
                use_maplit,
            )?;
        }
        Err(_) => {
            println!(
                "No canonical data for exercise '{}' found. Generating standard exercise template.",
                &exercise_name
            );

            test_file.write_all(b"// Add your tests here\n")?;
        }
    }

    generate_meta(&exercise_name, &exercise_path)?;
    generate_readme(&exercise_name)?;

    Ok(())
}