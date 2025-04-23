# pacpak
A pacman wrapper to manage installed flatpaks with pacman. This is still starting out and more of a proof of concept than a product.

## TODO
There is lots I want to add. (and maybe rewrite this in rust or c or sth)  
PR/issues are very welcome.  

I want to handle:
#### Q\[ilo\]
info/list files/owns file  
`flatpak [list|info] `  
`flatpak info --show-runtime --show-extensions <app-id>` (shows dependecies)  

#### R\[ns\]
`flatpak uninstall`
#### S\[yus\]
`flatpak update`
`flatpak search`
#### F?
not sure yet, I dont really use it  

### upstream
I think the flatpak cli can be better for interacting with a script, what is needed?  
- better `list`: json?, seperated list?
- `info`: list mutliple if unclear?
- list branches/arches of an appid
- show duplication of appid easily?
