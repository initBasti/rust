extern crate protein_translation as proteins;

#[test]
fn test_methionine() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.name_for("AUG"), Ok("methionine"));
}

#[test]
#[ignore]
fn test_cysteine_tgt() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.name_for("UGU"), Ok("cysteine"));
}

#[test]
#[ignore]
fn test_stop() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.name_for("UAA"), Ok("stop codon"));
}

#[test]
#[ignore]
fn test_valine() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.name_for("GUU"), Ok("valine"));
}

#[test]
#[ignore]
fn test_isoleucine() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.name_for("AUU"), Ok("isoleucine"));
}

#[test]
#[ignore]
fn test_arginine_name() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.name_for("CGA"), Ok("arginine"));
    assert_eq!(info.name_for("AGA"), Ok("arginine"));
    assert_eq!(info.name_for("AGG"), Ok("arginine"));
}

#[test]
#[ignore]
fn empty_is_invalid() {
    let info = proteins::parse(make_pairs());
    assert!(info.name_for("").is_err());
}

#[test]
#[ignore]
fn x_is_not_shorthand_so_is_invalid() {
    let info = proteins::parse(make_pairs());
    assert!(info.name_for("VWX").is_err());
}

#[test]
#[ignore]
fn too_short_is_invalid() {
    let info = proteins::parse(make_pairs());
    assert!(info.name_for("AU").is_err());
}

#[test]
#[ignore]
fn too_long_is_invalid() {
    let info = proteins::parse(make_pairs());
    assert!(info.name_for("ATTA").is_err());
}

#[test]
#[ignore]
fn test_translates_rna_strand_into_correct_protein() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.of_rna("AUGUUUUGG").unwrap(),
               vec!["methionine", "phenylalanine", "tryptophan"]);
}

#[test]
#[ignore]
fn test_stops_translation_if_stop_codon_present() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.of_rna("AUGUUUUAA").unwrap(),
               vec!["methionine", "phenylalanine"]);
}

#[test]
#[ignore]
fn test_stops_translation_of_longer_strand() {
    let info = proteins::parse(make_pairs());
    assert_eq!(info.of_rna("UGGUGUUAUUAAUGGUUU").unwrap(),
               vec!["tryptophan", "cysteine", "tyrosine"]);
}

#[test]
#[ignore]
fn test_invalid_codons() {
    let info = proteins::parse(make_pairs());
    assert!(info.of_rna("CARROT").is_err());
}

// The input data constructor. Returns a list of codon, name pairs.
fn make_pairs() -> Vec<(&'static str, &'static str)> {
    let grouped = vec![
        ("isoleucine", vec!["AUU", "AUC", "AUA"]),
        ("valine", vec!["GUU", "GUC", "GUA", "GUG"]),
        ("phenylalanine", vec!["UUU", "UUC"]),
        ("methionine", vec!["AUG"]),
        ("cysteine", vec!["UGU", "UGC"]),
        ("alanine", vec!["GCU", "GCC", "GCA", "GCG"]),
        ("glycine", vec!["GGU", "GGC", "GGA", "GGG"]),
        ("proline", vec!["CCU", "CCC", "CCA", "CCG"]),
        ("threonine", vec!["ACU", "ACC", "ACA", "ACG"]),
        ("serine", vec!["AGU", "AGC"]),
        ("tyrosine", vec!["UAU", "UAC"]),
        ("tryptophan", vec!["UGG"]),
        ("glutamine", vec!["CAA", "CAG"]),
        ("asparagine", vec!["AAU", "AAC"]),
        ("histidine", vec!["CAU", "CAC"]),
        ("glutamic acid", vec!["GAA", "GAG"]),
        ("aspartic acid", vec!["GAU", "GAC"]),
        ("lysine", vec!["AAA", "AAG"]),
        ("arginine", vec!["CGU", "CGC", "CGA", "CGG", "AGA", "AGG"]),
        ("stop codon", vec!["UAA", "UAG", "UGA"])];
    let mut pairs = Vec::<(&'static str, &'static str)>::new();
    for (name, codons) in grouped.into_iter() {
        for codon in codons {
            pairs.push((codon, name));
        }
    };
    pairs.sort_by(|&(_, a), &(_, b)| a.cmp(b));
    return pairs
}