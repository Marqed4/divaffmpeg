# Third-Party Notices

## FFmpeg

DivaFFMPEG calls the `ffmpeg` and `ffprobe` executables as separate programs. It does not link against FFmpeg libraries.

Release archives bundle an unmodified FFmpeg build:

- Build: 64-bit static Windows build from https://www.gyan.dev/ffmpeg/builds/ (`ffmpeg-git-full`)
- Version: `2026-03-18-git-106616f13d-full_build-www.gyan.dev`
- Configuration: `--enable-gpl --enable-version3` (no `--enable-nonfree`)
- License: GNU GPL version 3. Full text in `ffmpeg/LICENSE` inside the archive.
- Corresponding source: https://github.com/FFmpeg/FFmpeg/commit/106616f13d
- Build scripts and library versions: https://github.com/GyanD/codexffmpeg and `ffmpeg/README.txt` inside the archive.
- Upstream project: https://ffmpeg.org

The bundled build links many other libraries (x264, x265, libvpx, and others). Their licenses and sources are listed in `ffmpeg/README.txt` and the FFmpeg project documentation.

DivaFFMPEG (AGPL-3.0) and FFmpeg (GPLv3) are separate programs shipped together as an aggregate. GPLv3 section 13 allows combining GPLv3 and AGPLv3 works.

Nothing here replaces the license terms. Do not remove `ffmpeg/LICENSE` or `ffmpeg/README.txt` from the archive.

To use your own FFmpeg instead, put `ffmpeg.exe` and `ffprobe.exe` on your `PATH`.
