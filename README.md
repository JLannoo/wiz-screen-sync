# WiZ Screen Sync

A small program written in Rust.

It reads the pixels in your screen and sets your WiZ lightbulbs to the average.

Good for immersive gaming and content consumption.

## Instructions
Create a file called `ips.txt` in the same folder where the program is and set the IPs of the lamps you want to control in there.
Using the following format:
```
<FIRST_IP_HERE>
<SECOND_IP_HERE>
<THIRD_IP_HERE>
...
```

Press `ESC` to stop the program and restore the lightbuls to their previous setting.

### Info about WiZ lights' API from [pywizlight](https://github.com/sbidy/pywizlight/).