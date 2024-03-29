## wallpaper
窗口管理器上使用的壁纸工具

## 安装
首先你需要在系统上安装这些软件:
* openssl11-libs
* [xrandr >= 1.5.0](https://www.x.org/releases/X11R7.7/doc/man/man1/xrandr.1.xhtml)

设置壁纸功能:
* [feh >=3.4.1](https://feh.finalrewind.org/)

将视频设置壁纸的功能:
* [ffmpeg >=4.2.3](https://ffmpeg.org/)
* [imagemagick >=7.0.10.16](https://www.imagemagick.org/)

将GIF设置壁纸的功能:
* [imagemagick >=7.0.10.16](https://www.imagemagick.org/)

```
git clone https://github.com/smoothsea/wallpaper.git
cd wallpaper
cargo build --release
sudo cp ./target/release/wallpaper /usr/bin/
```

## 使用用例
* `wallpaper`
* 设置一个壁纸目录，自动切换壁纸的间隔时间为30秒: `wallpaper -d $HOME/.wallpaper -i 30`
* 下载壁纸: `wallpaper download --empty --sfw`
* 仅下载壁纸: `wallpaper -d $HOME/.wallpaper download --empty --only_download`
* 设置一个视频为壁纸: `wallpaper video -f video.mp4`
* 设置一张GIF为壁纸: `wallpaper gif -f test.gif`
* 更多: `wallpaper -h`
> 注：壁纸目录的默认值是`$HOME/.wallpaper`,默认下载的壁纸分辨率为当前显示器分辨率。


