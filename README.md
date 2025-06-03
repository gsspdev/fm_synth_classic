
# FM Synthesizer

A WebAssembly-powered FM synthesizer with a command-line interface that runs in your browser.

🎹 **[Try it live on GitHub Pages](https://gsspdev.github.io/fm_synth_classic/)**

## Features

- 12 FM synthesis presets (Bell, Bass, Electric Piano, etc.)
- 10 built-in melodies
- Terminal-style web interface
- Real-time audio synthesis using Web Audio API
- Written in Rust, compiled to WebAssembly

## Quick Start

### Online Demo

Visit the [live demo](https://gsspdev.github.io/fm_synth_classic/) to try the synthesizer immediately.

### Local Development

1. **Clone the repository**
   ```bash
   git clone https://github.com/YOUR_USERNAME/YOUR_REPO_NAME.git
   cd YOUR_REPO_NAME
   ```

2. **Install dependencies**
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Install wasm-pack
   curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
   ```

3. **Build the project**
   ```bash
   wasm-pack build --target web --out-dir pkg
   ```

4. **Serve locally**
   ```bash
   python3 -m http.server 8000
   # Open http://localhost:8000/index_wasm.html
   ```

## Commands

- `help` - Show available commands
- `list presets` - Show all 12 sound presets
- `list melodies` - Show all 10 melodies
- `play <preset> <melody>` - Play a melody with a preset
  - Example: `play bell twinkle`
  - Example: `play 1 3`
- `demo` - Play a demonstration
- `clear` - Clear the terminal

## GitHub Pages Deployment

This project includes automatic deployment to GitHub Pages using GitHub Actions.

### Setup

1. **Enable GitHub Pages in your repository**
   - Go to Settings → Pages
   - Under "Source", select "GitHub Actions"

2. **Push to main branch**
   - The workflow will automatically build and deploy your synthesizer

3. **Access your synthesizer**
   - Your synthesizer will be available at:
   - `https://YOUR_USERNAME.github.io/YOUR_REPO_NAME/`

### Manual Deployment

If you prefer to deploy manually:

```bash
# Build the project
wasm-pack build --target web --out-dir pkg

# Create a gh-pages branch
git checkout -b gh-pages

# Copy files
cp index_wasm.html index.html
git add pkg/ index.html debug.html
git commit -m "Deploy to GitHub Pages"
git push origin gh-pages
```

## Project Structure

```
.
├── .github/
│   └── workflows/
│       └── deploy.yml      # GitHub Actions workflow
├── src/
│   ├── lib.rs             # WebAssembly library
│   └── main.rs            # Desktop version (optional)
├── Cargo.toml             # Rust dependencies
├── index_wasm.html        # Web interface
├── debug.html             # Debug interface
└── build.sh               # Build script
```

## Troubleshooting

### No sound?
- Click anywhere on the page first (browser security requirement)
- Check your system volume
- Try the debug page: `/debug.html`

### Module won't load?
- Check browser console for errors (F12)
- Ensure you're using a modern browser (Chrome, Firefox, Edge)
- Verify the pkg/ directory was created during build

### Build errors?
- Update Rust: `rustup update`
- Clear cache: `rm -rf target/ pkg/`
- Check you have wasm32 target: `rustup target add wasm32-unknown-unknown`

## Browser Support

- Chrome/Edge 66+
- Firefox 60+
- Safari 14+
- Mobile browsers with Web Audio API support

## License

MIT License - feel free to use this project for learning or in your own applications.

## Credits

Built with:
- [Rust](https://www.rust-lang.org/)
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
- [Web Audio API](https://developer.mozilla.org/en-US/docs/Web/API/Web_Audio_API)
