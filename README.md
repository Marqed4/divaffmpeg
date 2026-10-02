<div align="center">
  <br/>
    <img src="/examples/diva_icon.gif" title="DivaFFMPEG" alt="DivaFFMPEG logo" width="200" />
    <h1>𝐷𝑖𝑣𝑎 𝐹𝐹𝑀𝑃𝐸𝐺</h1>
    <p>A terminal UI wrapper around ffmpeg, because typing flags is beneath her</p>
    <p>Convert, compress, trim, and merge your video. Point, select, done.</p>
    <p align="center"><img src="/examples/frontpage.gif" /></p>
    <br/>
</div>

---

<h2 align="center">Views</h2>

|  |
|---|
| <p align="center"><strong>Image processing</strong><br/>In progress.</p> |
| <p align="center"><img src="H;/image_placeholder5" /></p> |
| <p align="center"><strong>Video processing</strong><br/>Convert, compress, trim, merge, all in one place.</p> |
| <p align="center"><img src="/examples/video_tools.gif" /></p> |
| <p align="center"><strong>Audio processing</strong><br/>In progress.</p> |
| <p align="center"><img src="H;/image_placeholder4" /></p> |

---

<h2 align="center">Features</h2>

<div align="left">

- **Video conversion**
  Swap formats without memorizing codec incantations.

- **Video compression**
  Real compress slider, live progress bar, no guessing.

- **Video trim and merge**
  Cut what you don't need, stitch together what you do.

- **Resolution and FPS presets**
  360 to 4k, and 24fps to 120fps.

- **Image processing** (in progress)
  Not there yet.

- **Audio processing** (in progress)
  Not there yet.

</div>

---

<h2 align="center">Setup</h2>

DivaFFMPEG needs FFmpeg, which is too big to live in the repo. After cloning:

1. Download `ffmpeg-win64.zip` from the [latest release](https://github.com/Marqed4/DivaFFMPEG/releases/latest).
2. Unzip it into the repo root, so you end up with `ffmpeg/bin/ffmpeg.exe` and `ffmpeg/bin/ffprobe.exe`.
3. `cargo run`

Skip this if `ffmpeg` and `ffprobe` are already on your `PATH`.

---

<h2 align="center">Ethics</h2>

DivaFFMPEG takes zero liability for whatever you point it at. Your files, your choices.

---

<h2 align="center">Contributing</h2>
- If you're considering contributing to DivaFFMPEG, thank you, right this way.
- Check the [Contribution Guidelines](CONTRIBUTING.md) before getting started. Opinions are welcomed.

---

## License

This project is licensed under the **AGPL-3.0 License** - see the [LICENSE](LICENSE) file for details.

*Release bundles include an unmodified FFmpeg build (64-bit static Windows build from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/), `full_build`, git `106616f13d`), configured with `--enable-gpl --enable-version3` and therefore licensed under the [GPLv3](https://www.gnu.org/licenses/gpl-3.0.html). FFmpeg is a trademark of Fabrice Bellard, originator of the FFmpeg project. DivaFFMPEG is not affiliated with or endorsed by the FFmpeg project. See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for license text and source code.*
