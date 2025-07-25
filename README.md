# pacpak
[![License](https://img.shields.io/github/license/egnrse/pacpak)](https://github.com/egnrse/pacpak/blob/main/LICENSE)
[![GitHub last commit (branch)](https://img.shields.io/github/last-commit/egnrse/pacpak/main)](https://github.com/egnrse/pacpak/commits/main)
[![GitHub tag (latest SemVer pre-release)](https://img.shields.io/github/v/tag/egnrse/pacpak?label=version)](https://github.com/egnrse/pacpak/releases)

A pacman wrapper to manage installed flatpaks with pacman. This is still starting out.  

(I am currently rewriting this in rust. Look at the other branch if interested.)

## Install
Build using cargo (in the root directory of this project):
```bash
export CARGO_TARGET_DIR=target
cargo build --locked
```
Install either using cargo or copy `./target/debug/pacpak` into your path manually.

To install the deprecated bash branch, just put `pacpak.sh` into your path as `pacpak` and make it executable.  

There might also be packages for you:

[![Packaging status](https://repology.org/badge/vertical-allrepos/pacpak.svg)](https://repology.org/project/pacpak/versions)

## Usage
Use it just like `pacman` (**Not all args are supported!**)  
(There are some settings that can be customized. See `struct Config`.) 


## TODO
There is lots I want to add. PR/issues are very welcome.  
to implement:  
- Qo (file owned by what package) - works kinda
- Ql (list package files) - works
- Q\[u|t|s|\] (can be upgraded|orphans|search names/descriptions)
- S  (install) - works (prefers pacman over flatpaks)
- S\[s|u\] (search packages|upgrade)
- Si    (package information (from online?))
- R\[s|n\] (remove: also dependecies|remove config files) - R works (is currenly doing Rs)
- deal with fullnames of operations (eg. --query) - works through clap (I think)
- some more tricky Qi-fields
- cache for `flatpak list|info` (paths?) to speed things up (prob best to write a libray to directly interface with OSTree)

I want to handle:
#### Q\[ilo\]
info/list files/owns file  
`flatpak [list|info]`  
`flatpak info --show-runtime --show-extensions <app-id>` (shows dependecies)  

#### R\[ns\]
`flatpak uninstall`  
#### S\[yus\]
`flatpak update`  
`flatpak search`    (very slow)  
#### F?
not sure yet, I dont really use it  
#### V

### upstream?
I think the flatpak cli can be better for interacting with a script, what is needed?  
- better `list`: json?, seperated list?
- `info`: list multiple if unclear? or make more script friendly
- list branches/arches of an appid
- list extended appID (appID/branch/arch)?
- show duplication of appid easily?
- search could be faster: indexing, local first?, only local option?
