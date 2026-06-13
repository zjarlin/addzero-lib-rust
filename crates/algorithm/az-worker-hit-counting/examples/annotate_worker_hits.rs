use az_worker_hit_counting::logic_worker_hit_counting::assist::annotate_worker_hits_video;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let video_path = std::env::args()
        .nth(1)
        .ok_or("missing absolute video path")?;
    let annotated_video = annotate_worker_hits_video(video_path)?;
    println!("{}", annotated_video.display());
    Ok(())
}
