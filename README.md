# pacpak
[![License](https://img.shields.io/github/license/egnrse/pacpak)](https://github.com/egnrse/pacpak/blob/main/LICENSE)
[![GitHub last commit (branch)](https://img.shields.io/github/last-commit/egnrse/pacpak/main)](https://github.com/egnrse/pacpak/commits/main)
[![GitHub tag (latest SemVer pre-release)](https://img.shields.io/github/v/tag/egnrse/pacpak?label=version)](https://github.com/egnrse/pacpak/releases)

A pacman wrapper to manage installed flatpaks with pacman. This is still starting out and more of a proof of concept than a fully functional program.  

(I am currently rewriting this in rust. Look at the other branch if interested.)

## Install
For right now, just put `pacpak.sh` into your path as `pacpak` and make it executable.  

There might also be packages for you:

[![Packaging status](https://repology.org/badge/vertical-allrepos/pacpak.svg)](https://repology.org/project/pacpak/versions)

## Usage
Use it just like `pacman` (**Not all args are yet supported!** But the handing arguments through to pacman should still work fine)  
(There are some settings at the top of `pacpak.sh`, that can be customized) 


## TODO
There is lots I want to add. PR/issues are very welcome.  
to implement:  
- Qo (file owned by what package)
- Ql (list package files)
- Q\[u|t|s|\] (can be upgraded|orphans|search names/descriptions)
- S  (install)
- S\[s|u\] (search packages|upgrade)
- Si    (package information (from online?))
- R\[s|n\] (remove: also dependecies|remove config files)
- deal with fullnames of operations (eg. --query)
- cache for `flatpak list|info` (paths?) to speed things up
- some more tricky Qi-fields

- (and maybe rewrite this in rust or c or sth, to gain much needed speed ups [might be having to little time for that though])  

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
