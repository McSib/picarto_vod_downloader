# Picarto VOD Downloader
This is a basic downloader made for downloading Picarto VODs. It is meant to be faster than [youtube-dl](https://github.com/ytdl-org/youtube-dl "youtube-dl") by being meant only for Picarto, while youtube-dl is meant for a wide range of sites.

## Current state of the program
This program is extremely messy with code sitting in only the main fuction. I intended for this software to originally be a small and basic program, but after working on it, it's clear that there was a lot more I wanted to add. Because of this, I'm going to be updating and arranging the code a little better. The first steps being more readable and managable code.

## Requirements
***This program is under developement and expects a lot from the user right now.*** I want to emphasize this encase you start wondering why it doesn't work when you run it. First of all, this program is expecting three things: 
- That you have Windows
- That you have FFmpeg in your `PATH`
- That you have .NET installed

Since this was originally developed for my own purposes, I made it to fit to my purposes. I have FFmpeg in my `PATH`, and I work mainly on Windows. I may take a look at doing a Linux version, as it wouldn't be hard at all, but the main issue is that I have Windows specific code inside, making calls to the PowerShell to create a merge file for FFmpeg. So if I could get code to get around this, I may try something.

Another requirement is that you have .NET installed, my software uses the PowerShell and makes calls to .NET for encoding a text file to UTF-8 (since the default encoding for `echo "file 'INPUT_HERE'" >> merge.txt` breaks FFmpeg quite abruptly), so because of that, you will need .NET until I can find a better method (more than likely having the merge file generate internally without PowerShell.)

## Plans After Cleanup
Once the program is finished being cleaned up and given a new and better coat of paint, I will take my time to give this README a better description as well as a tutorial on how to use it. But for right now, the README will remain light and straight to the point.
