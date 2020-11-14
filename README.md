## wallpaper
一个在窗口管理器上使用的壁纸管理工具。可以下载，自动切换，将视频设置为壁纸。自适应多显示器。

## 安装
首先你需要在系统上安装这些软件:
* openssl11-libs
* [xrandr >= 1.5.0](https://www.x.org/releases/X11R7.7/doc/man/man1/xrandr.1.xhtml)

设置壁纸功能:
* [feh >=3.4.1](https://feh.finalrewind.org/)

将视频设置壁纸的功能:
* [ffmpeg >=4.2.3](https://ffmpeg.org/)
* [imagemagick >=7.0.10.16](https://www.imagemagick.org/)
* [xrandr >=1.5.1](https://gitlab.freedesktop.org/xorg/app/xrandr)
* [xdg-utils >=1.1.3](https://www.freedesktop.org/wiki/Software/xdg-utils/)
* [bash >=4.0](http://tiswww.case.edu/php/chet/bash/bashtop.html)
* [sed >=4.5](http://sed.sourceforge.net/) 

> 有些软件的版本可能低于上面写的的版本。

```
git clone https://github.com/smoothsea/wallpaper.git
cd wallpaper
cargo build --release
sudo cp ./target/release/wallpaper /usr/bin/
```

## 卸载
```
sudo rm /usr/bin/wallpaper
```

## 使用用例
* 最简单的: `wallpaper`
* 设置一个壁纸目录，自动切换壁纸的间隔时间为30秒: `wallpaper -d $HOME/.wallpaper -i 30`
* 自动下载壁纸: `wallpaper download --empty --sfw`
* 仅下载壁纸: `wallpaper -d $HOME/.wallpaper download --empty --only_download`
* 设置一个视频为壁纸: `wallpaper video -f video.mp4`
* 更多: `wallpaper -h`
> 注：壁纸目录的默认值是`$HOME/.wallpaper`,默认下载的壁纸分辨率为当前显示器分辨率。下载壁纸可能有些慢，因为源是国外的站点。


