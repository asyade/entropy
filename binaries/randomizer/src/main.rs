use std::path::Path;
use api_connector::stable_diffusion::*;
use api_connector::openai::*;
use api_connector::keyring::*;
use clap::Parser;
use prompt_generator::PromptGenerator;

#[derive(Parser, Debug)]
struct Args {
    data_path: String,
    template_path: String,
}

static NEGATIVE_PROMPT: &str = "(title), (text), ((((underage)))), ((((child)))), (((kid))), (((preteen))), ((((frame)))), ((((border)))), (((((background))))), ((tiling)), poorly drawn hands, poorly drawn feet, poorly drawn face, out of frame, extra limbs, deformed, body out of frame, bad anatomy, watermark, signature, cut off, low contrast, underexposed, overexposed, bad art, beginner, amateur, distorted face, blurry, draft, grainy";

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv().ok();
    
    let keys = KeyChain::from_env();

    let args = Args::parse();
    let generator = PromptGenerator::new(Path::new(&args.template_path), Path::new(&args.data_path)).unwrap();

    let prompt = generator.randomize();
    println!("{}", &prompt.render);
    let response = StableDiffusionConnector::new(&keys).generate_image(prompt.render, Some(String::from(NEGATIVE_PROMPT)), None).await.unwrap();
    dbg!(response);
}
