# ha-agent-rs [![CI/CD](https://github.com/Alekzanther/ha-agent-rs/actions/workflows/ci-cd.yml/badge.svg)](https://github.com/Alekzanther/ha-agent-rs/actions/workflows/ci-cd.yml)

Hello there, code wranglers, digital cowboys, and Internet dwellers! Welcome to the whimsical, fantastical, and, most importantly, **BLAZINGLY FAST** world of `ha-agent-rs`!

## What is ha-agent-rs? 🚀

Grab a chair, my friend! `ha-agent-rs` is a high-octane, Rust-infused concoction that I mixed up in my virtual code-lab. It's like the secret sauce in your favorite sandwich, but for Home Assistant.

At its core, `ha-agent-rs` is a beautiful, albeit slightly mad, piece of work that communicates the state of your webcam and microphone to your Home Assistant setup. It's currently at its adorable fledgling stage, version 0.x, but it works like a charm (on Linux).

Imagine having an automated party mode that kicks in when your webcam turns off, or a "Do Not Disturb" sign that lights up when your mic is active. `ha-agent-rs` makes it possible, because it loves nothing more than to keep a watchful eye on your mic and webcam and report back to Home Assistant. Talk about loyalty, eh?

## I'm Intrigued! How Do I Use It? 💻

I see I've piqued your interest! Here's how you can join in on the fun:

### Method 1: Install from crates.io

This method is super easy. Open your terminal and type:

```shell
cargo install ha-agent-rs
```

### Method 2: Build from GitHub

**Step 1:** Clone this repository. It's as simple as:

```shell
git clone https://github.com/yourusername/ha-agent-rs.git
```

**Step 2:** Move to the project directory:

```shell
cd ha-agent-rs
```

**Step 3:** We use environment variables to store sensitive information. I've added a .env-example file as a placeholder. Copy it to a new .env file:

```shell
cp .env-example .env
```
Then fill it in with your actual data. No peeking, please!

**Step 4:** Time to get this party started:

```shell
cargo run
```
### Configuration

In order to run it you need to supply it with the URL and long lived access token. Here are 3 different methods:

```Command line arguments
export HASS_URL="your_home_assistant_url"
export HASS_TOKEN="your_home_assistant_token"
```

```Command line arguments
ha-agent-rs --url "your_home_assistant_url" --token "your_home_assistant_token"
```

```.env file
cp .env-example .env
```

For more information on how to retrieve a long lived access token, see https://www.home-assistant.io/docs/authentication/#your-account-profile .

## What's Next? 🚀

This is just the beginning of ha-agent-rs. The future holds more features, more refinements, and more dad jokes!

## Join the Adventure! 🎉

Do you enjoy coding, automation, and slightly exaggerated product descriptions? Then we're going to get along just fine! Feel free to open an issue, submit a pull request, or share your thoughts.

## Lastly...

Remember, they used to call people who wrote in binary, wizards. Now we call them programmers. But with ha-agent-rs, you can be both!

**One day or day one. You decide. Happy coding!** 🎉🎉🎉
