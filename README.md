# Stilla

Stilla is a static site generator built in Rust. It is heavily inspired by [Zola](https://www.getzola.org/), but is slightly more opinionated focuses on simplicity rather than flexibility.

**Important Disclaimer:** I built Stilla purely to learn more about Rust, I don't recommend using it in production and I give no guarantees that it works as expected on all platforms. If you're looking for a production ready SSG, take a look at Zola or any of the ones listed at https://jamstack.org/generators/.

## Features

- Uses Markdown to generate pages
- Built in [Tailwind](https://tailwindcss.com/) Support
- Drafts - prepend an .md file with `#` to avoid including it in the build
- Date on filenames - prepend a file with a date and it will be accessible in templates (`2022-11-17_my-post.md`)

## How to use

Stilla relies on two directories for building your site: `pages` and `templates`.

### Pages

This is where your markdown content lives.

**Caveat:** Stilla currently only (reliably) supports a single level for subpages. i.e `mysite/posts/my-post`. Feel free to commit a solution for deeper levels!

## About the name

"Stilla" is Swedish for "Still", "Calm" or "Steady", which I felt fit nicely to a static site generator!
