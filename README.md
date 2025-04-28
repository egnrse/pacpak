# pacpak
A pacman wrapper to manage installed flatpaks with pacman. This is still starting out and more of a proof of concept than a fully functional program.  

## TODO
There is lots I want to add. PR/issues are very welcome.  
implemented:  
- Qo (file owned by what package)
- Ql (list package files)
- Q\[u|t|s|\] (can be upgraded|orphans|search names/descriptions)
- S  (install)
- S\[s|u\] (search packages|upgrade)
- Si    (package information (from online?))
- R\[s|n\] (remove: also dependecies|remove config files)
- deal with fullnames of operations (eg. --query)
- cache for `flatpak list|info` (paths?) to speed things up

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
