# Joinerator
A utility for "stylizing" text with Unicode combining marks.

## What does it do?
Unicode has entire sections dedicated to combining marks.
This wonderful little tool offers a variety of ways to randomly add said combining marks to any string.

## What does it actually help me with?
Ever wanted a fancy username for social media?  
`N͋ͅo̜w̫ͦ yŏu͇ c̤̈ȁ̞n e̜a͐s̼iͮḻͯŷ̥ ha͔̔v̑eͣ o̹n̈è!`

If you can think of anything practical to use this for, be my guest.

## Command Line
Every supported option can be seen with `joinerator --help`, but here's a quick start guide you can use:

**Automatically joinerate everything you copy into your clipboard:**  
It's great for annoying your friends with unreadable text!
```bash
joinerator -i clipboard -o clipboard --watch
```

**Magically joinerate your source code and make it impossible to compile:**  
You can use it like any other Unix command line tool.
```bash
cat src/main.rs | joinerator
```

**Change the frequency of how often combining marks appear:**  
You can change how often the marks appear, and how many marks can stack at once.

```bash
joinerator --above:stacking 2 --above:frequency 50%
```
