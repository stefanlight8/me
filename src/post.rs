use {
    chrono::NaiveDate,
    std::{error::Error, fmt, path::PathBuf},
    tokio::{
        fs::File,
        io::{self, AsyncBufReadExt, AsyncReadExt, BufReader, Lines},
    },
};

async fn next_line(
    lines: &mut Lines<&mut BufReader<File>>,
) -> Result<Option<String>, PostParsingError> {
    lines.next_line().await.map_err(PostParsingError::Io)
}

#[derive(Debug)]
pub enum PostParsingError {
    Io(io::Error),
    NoHeader,
    TitleMissing,
    DescriptionMissing,
    CreatedMissing,
    DateParseError(chrono::ParseError),
    NotFound,
}

impl Error for PostParsingError {}

impl fmt::Display for PostParsingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostParsingError::Io(err) => write!(f, "failed to read post: {:?}", err),
            PostParsingError::NoHeader => write!(f, "failed to parse post: no header"),
            PostParsingError::TitleMissing => {
                write!(f, "failed to parse post header: title is missing")
            }
            PostParsingError::DescriptionMissing => write!(
                f,
                "failed to parse post header: failed to parse post: description is missing"
            ),
            PostParsingError::CreatedMissing => {
                write!(f, "failed to parse post header: created is missing")
            }
            PostParsingError::DateParseError(err) => write!(
                f,
                "failed to parse post header: failed to parse date: {:?}",
                err
            ),
            PostParsingError::NotFound => write!(f, "failed to parse post: file not found"),
        }
    }
}

pub struct PostHeader {
    pub title: String,
    pub description: String,
    pub created: NaiveDate,
}

impl PostHeader {
    pub async fn parse(buffer: &mut BufReader<File>) -> Result<PostHeader, PostParsingError> {
        let mut lines = buffer.lines();

        if next_line(&mut lines).await?.as_deref() != Some("---") {
            return Err(PostParsingError::NoHeader);
        }

        let mut title: Option<String> = None;
        let mut description: Option<String> = None;
        let mut created: Option<NaiveDate> = None;

        while let Some(line) = next_line(&mut lines).await? {
            let line = line.trim();

            if line == "---" {
                return Ok(PostHeader {
                    title: title.ok_or(PostParsingError::TitleMissing)?,
                    description: description.ok_or(PostParsingError::DescriptionMissing)?,
                    created: created.ok_or(PostParsingError::CreatedMissing)?,
                });
            }

            if let Some((key, value)) = line.split_once(':') {
                let value = value.trim();

                match key {
                    "title" => title = Some(value.to_string()),
                    "description" => description = Some(value.to_string()),
                    "created" => {
                        created = Some(
                            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                                .map_err(|err| PostParsingError::DateParseError(err))?,
                        )
                    }
                    key => tracing::warn!("{} is unparsed", key),
                }
            }
        }

        Err(PostParsingError::NoHeader)
    }
}

pub struct Post {
    pub header: PostHeader,
    pub content: String,
}

impl Post {
    pub async fn parse(file: PathBuf) -> Result<Post, PostParsingError> {
        let file = File::open(file)
            .await
            .map_err(|err| PostParsingError::Io(err))?;
        let mut buffer = BufReader::new(file);

        let mut post = Post {
            header: PostHeader::parse(&mut buffer).await?,
            content: String::new(),
        };

        buffer
            .read_to_string(&mut post.content)
            .await
            .map_err(|err| PostParsingError::Io(err))?;

        Ok(post)
    }
}

pub async fn get_post(posts_path: PathBuf, slug: &str) -> Result<Post, PostParsingError> {
    let path = posts_path.join(format!("{}.md", slug));

    if !path.exists() {
        return Err(PostParsingError::NotFound);
    }

    Post::parse(path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_post_parse() {
        let path = PathBuf::from("test_post.md");

        let mut file = tokio::fs::File::create(&path)
            .await
            .expect("create test file");

        let content = r#"---
title: Hello World
description: Test post
created: 2026-04-26
---
This is the content of the post.
It has multiple lines.
"#;

        file.write_all(content.as_bytes())
            .await
            .expect("write test file");

        drop(file);

        let post = Post::parse(path.clone()).await.unwrap();

        assert_eq!(post.header.title, "Hello World");
        assert_eq!(post.header.description, "Test post");
        assert_eq!(
            post.header.created,
            NaiveDate::from_ymd_opt(2026, 4, 26).unwrap()
        );
        assert!(post.content.contains("This is the content"));
        assert!(post.content.contains("multiple lines"));

        let _ = tokio::fs::remove_file(path).await;
    }
}
