// use git2::{Direction, Remote};

fn main() -> Result<(), anyhow::Error> {
    Ok(())
}

// fn main() -> Result<(), git2::Error> {
//     let mut remote = Remote::create_detached("https://github.com/golang/go")?;

//     // let mut fetch_options = FetchOptions::new();
//     // fetch_options.download_tags(git2::AutotagOption::All);
//     // fetch_options.custom_headers(&["--sort=version:refname"]);

//     remote.connect(Direction::Fetch)?;
//     // remote.fetch::<&str>(&[], Some(&mut fetch_options), None)?;

//     for remote_ref in remote.list()? {
//         println!("{}", remote_ref.name());
//     }

//     remote.disconnect()?;

//     Ok(())
// }
