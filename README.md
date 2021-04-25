# Septem
A weekly productivity tracker for Unix-like systems running Xorg.

### Current Plan
As of right now, I plan on making a simple application that tracks what application
is the current active window using xcb and EWMH, since I am running bspwm with no DE.
Then, you configure a whitelist of applications that are seen as "productive" for each day of the week.
Septem then keeps track of how long you're on these or not to monitor how "productive" you are.
At the end of each week, it then produces a file that give a general percentage of how productive
you were for the week. As well, it has the percentage for each individual application.

### Current Goals
- [x] Handle change of active window
- [x] Track what application is active via PID and WM Name
- [ ] Keep record of how long and the percentage for each apps 
- [ ] Create weekly config system (maybe .toml files)
- [ ] Create alert system for being unproductive
- [ ] Ability to enable for certain times of day (ie 9-5 work hours)
- [ ] Ability to disable on for certain days of the year 
  - Certain Number Day like Christmas or New Year's Eve
  - Certain Day/Month combo like Thanksgiving or Election Day
- [ ] Multiple Screen support


### Future Developments
- [ ] Tray Icon/Support to quickly turn on and off (like redshift-gtk)
- [ ] Additional abilities for alert system like telling you to take breaks 
- [ ] Simple graphical application that can generate config files and visualize data
- [ ] Startup service support for bsd init system (maybe Linux systemd but I can't test it)
- [ ] Track and whitelist webpages you're on (Probably just for Firefox)
- [ ] Can be reconfigured whilst the program is running
- [ ] Support for wayland/SwayWM (or seperate version) 

### Rust Libraries being used
- [xcb](https://github.com/rtbo/rust-xcb) by Remi Thebault - [@rtbo](https://github.com/rtbo)
- [xcb-util](https://github.com/meh/rust-xcb-util) by A Depressed Man - [@meh](https://github.com/meh)
