use biomedgps::proxy::website::{format_sanger_cosmic_target_url, Website};
use poem::web;

fn main() {
    let website = Website {
        name: "sanger_cosmic",
        base_url: "https://cancer.sanger.ac.uk",
        redirect_url: "https://omics-data.3steps.cn/sanger_cosmic",
        target_url: "https://cancer.sanger.ac.uk/cosmic/gene/analysis?ln=",
        format_target_url: format_sanger_cosmic_target_url,
        which_tags: vec!["a", "link", "script", "img", "[data-url]", "table[title]"],
    };
    let input = r#"
        <div>
            <a href="/about">About</a>
            <a href="/contact">Contact</a>
            <link rel="stylesheet" href="style.css">
            <script src="/script.js"></script>
            <img src="/logo.png">
            <img src="/banner.jpg">
            <div data-url="/page"></div>
            <div data-url="https://example.com/page"></div>
            <table title="/table-page"></table>
            <table title="https://example.com/table-page"></table>
        </div>
    "#;

    let modified_html_v1 = match website.modify_html(input, website.which_tags.clone()) {
        Ok(modified_html) => modified_html,
        Err(e) => {
            eprintln!("Error: {}", e);
            return;
        }
    };
    println!("{}", modified_html_v1);

    let modified_html_v2 =
        match website.modify_html(input, website.which_tags.clone()[0..3].to_vec()) {
            Ok(modified_html) => modified_html,
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        };
    println!("{}", modified_html_v2);
}
