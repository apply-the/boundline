# Installation

This page covers how to install the Boundline CLI across different platforms.

Boundline is shipped alongside [Canon](https://apply-the.github.io/canon).

### Linux (Ubuntu / Debian)

We provide official `.deb` packages for `amd64` and `arm64` via the _Apply The_ [APT](https://github.com/apply-the/packages) repository.

```bash
curl -fsSL https://apply-the.github.io/packages/apt/gpg.key \
  | sudo gpg --dearmor -o /usr/share/keyrings/apply-the-archive-keyring.gpg

echo "deb [signed-by=/usr/share/keyrings/apply-the-archive-keyring.gpg] https://apply-the.github.io/packages/apt stable main" \
  | sudo tee /etc/apt/sources.list.d/apply-the.list

sudo apt update
sudo apt install boundline
```

**Later updates:**
```bash
sudo apt update
sudo apt upgrade boundline
```

### MacOS

Installation requires _Homebrew_:

```bash
brew tap apply-the/boundline
brew install boundline
```

**Later updates:**
```bash
brew update
brew upgrade boundline
```

### Windows

Use the published winget package:

```powershell
winget install ApplyThe.Boundline
```

### Local development

For local development, unreleased validation, or when release channels are not
available, you can build from source using Cargo:

```bash
git clone https://github.com/apply-the/boundline.git
cd boundline
cargo install --path .
```

> [!TIP]
> After any installation method, run `boundline doctor --install` to verify the installed Boundline executable and the documented Canon pairing for the current release line.