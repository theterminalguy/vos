# Chapter 1: Hello, VOS! 👋

**Welcome to your journey of understanding how computers really work!**

## 🎯 What You'll Learn

In this chapter, you'll:
- Understand what VOS is (in simple terms!)
- Start the VOS shell for the first time
- Run your first commands
- Create files and directories
- Feel like a computer wizard 🧙‍♂️

**Time needed:** 15 minutes
**Difficulty:** Easy - No prior knowledge needed!

---

## 🤔 What is VOS?

Imagine you have a toy computer inside your real computer. That's VOS!

### The Simple Answer

**VOS = A pretend computer that works just like a real one**

It has everything a real computer has:
- A brain (CPU) that thinks
- Memory (RAM) that remembers things
- A hard drive (filesystem) that stores files
- Programs that do stuff

### Why Does This Matter?

Learning VOS is like learning to drive with a simulator:
- ✅ You can't break anything!
- ✅ You can see how everything works inside
- ✅ You learn real skills
- ✅ It's actually fun!

---

## 🏗️ The Computer Town Analogy

Let's understand a computer by imagining a town:

```
    🏛️ COMPUTER TOWN 🏛️

    ┌─────────────────────────────┐
    │  City Hall (CPU)            │ ← The mayor who makes all decisions
    │  "I run instructions!"      │
    └─────────────────────────────┘
              │
              ↓
    ┌─────────────────────────────┐
    │  Library (Memory/RAM)       │ ← Stores information temporarily
    │  "I remember things!"       │
    └─────────────────────────────┘
              │
              ↓
    ┌─────────────────────────────┐
    │  Archive (Filesystem)       │ ← Stores files permanently
    │  "I keep your files safe!"  │
    └─────────────────────────────┘
              │
              ↓
    ┌─────────────────────────────┐
    │  Town Square (Shell)        │ ← Where you talk to the computer
    │  "I'm your interface!"      │
    └─────────────────────────────┘
```

When you type a command:
1. You speak at the Town Square (shell)
2. The message goes to City Hall (CPU)
3. City Hall checks the Library (memory) if needed
4. Files are saved in the Archive (filesystem)
5. You get a response!

---

## 🚀 Let's Start VOS!

### Step 1: Open Your Terminal

On **Mac**: Press `Cmd + Space`, type "Terminal", press Enter
On **Linux**: Press `Ctrl + Alt + T`
On **Windows**: Use WSL or Git Bash

### Step 2: Go to the VOS Directory

```bash
cd ~/code/vos
```

*If you haven't downloaded VOS yet, check the [QUICKSTART.md](../../QUICKSTART.md) first!*

### Step 3: Launch VOS

```bash
cargo run --release --bin vos-cli
```

Wait a moment (it's starting up the computer!)...

You should see:

```
VOS Shell v0.1.0
Type 'help' for available commands, 'exit' to quit.

vos>
```

🎉 **Success!** That blinking cursor `vos>` is waiting for you!

---

## 💬 Your First Conversation with VOS

Let's talk to VOS! Type each command and press Enter.

### Command 1: Say Hello

```bash
vos> echo Hello, VOS!
```

**What happens:**
```
Hello, VOS!
vos>
```

VOS repeated what you said! The `echo` command is like asking VOS to speak.

```
You: "Echo 'Hello, VOS!'"
       ↓
VOS: "Hello, VOS!" (repeats it back)
```

### Command 2: Ask for Help

```bash
vos> help
```

**What happens:**
You'll see a list of all commands VOS understands!

```
VOS Shell - Available Commands:

  help         Display this help message
  exit         Exit the shell
  pwd          Print working directory
  cd [PATH]    Change directory
  ls [PATH]    List directory contents
  cat FILE     Display file contents
  echo [ARGS]  Echo arguments to output
  mkdir PATH   Create directory
  touch FILE   Create empty file
  rm PATH      Remove file or empty directory
```

Think of these as VOS's vocabulary - the words it knows!

### Command 3: Where Am I?

```bash
vos> pwd
```

**What happens:**
```
/ (inode 1)
vos>
```

`pwd` means "Print Working Directory" - it tells you where you are in the computer's filesystem.

The `/` symbol means "the root" - the very top of the file tree, like the entrance to a building.

```
    / (You are here!)
    └── Everything else is inside here
```

---

## 📁 Creating Your Digital Space

Now let's create some files and folders! This is where it gets fun.

### Understanding Directories

A **directory** is just a fancy word for "folder" - a place to put files.

Think of it like organizing a desk:
```
Your Desk
├── Documents folder 📁
├── Photos folder 📁
└── Projects folder 📁
    └── VOS folder 📁 (folder inside folder!)
```

### Create Your First Directory

```bash
vos> mkdir /myspace
```

**What happened:**
- `mkdir` = "make directory" (create a folder)
- `/myspace` = the name of your new folder
- VOS just created a folder called "myspace" at the root (`/`)

**Visual:**
```
Before:          After:
/                /
                 └── myspace/  ← New!
```

No output means success! (Unix style - if there's no error, it worked)

### Check That It Exists

```bash
vos> ls /
```

**Output:**
```
myspace  (inode 2)
vos>
```

`ls` means "list" - show me what's in this folder!

You can see your `myspace` directory!
*(Don't worry about "inode 2" for now - that's internal stuff)*

### Go Inside Your Directory

```bash
vos> cd /myspace
```

**What happened:**
- `cd` = "change directory" (move to a different folder)
- You're now "inside" the myspace folder!

Think of it like walking into a room.

### Confirm You Moved

```bash
vos> pwd
```

**Output:**
```
/ (inode 2)
vos>
```

The inode changed! You're now inside `/myspace` (inode 2), not at root (inode 1).

---

## 📝 Creating Files

Now let's create some files!

### Create Your First File

```bash
vos> touch hello.txt
```

**What happened:**
- `touch` creates an empty file
- `hello.txt` is the filename
- You just created a text file!

### Create More Files

```bash
vos> touch notes.txt
vos> touch ideas.txt
```

### List Your Files

```bash
vos> ls
```

**Output:**
```
hello.txt  (inode 3)
notes.txt  (inode 4)
ideas.txt  (inode 5)
vos>
```

🎉 Look at that! You have three files now!

---

## 🌳 Building a File Tree

Let's create a more complex structure:

```bash
vos> mkdir documents
vos> mkdir projects
vos> mkdir projects/vos-demo
vos> touch projects/vos-demo/readme.txt
```

**What you just built:**
```
/myspace/
├── documents/
├── projects/
│   └── vos-demo/
│       └── readme.txt
├── hello.txt
├── notes.txt
└── ideas.txt
```

### Navigate Your Tree

```bash
# Go into projects
vos> cd projects

# Where am I?
vos> pwd

# What's here?
vos> ls

# Go deeper
vos> cd vos-demo

# What's here?
vos> ls

# Go back up one level
vos> cd ..

# Go back to root
vos> cd /
```

**The `..` Trick:**
- `.` = current directory ("here")
- `..` = parent directory ("one level up")

Think of it like:
- `.` = "this room"
- `..` = "the hallway outside"

---

## 🎯 Interactive Challenge!

**Mission:** Create this exact structure:

```
/
└── home/
    └── yourname/
        ├── work/
        │   └── project.txt
        ├── personal/
        │   └── notes.txt
        └── readme.txt
```

**Steps to try:**
<details>
<summary>💡 Hint #1 (click to reveal)</summary>

Start by creating the `/home` directory:
```bash
vos> mkdir /home
```
</details>

<details>
<summary>💡 Hint #2</summary>

Then create a directory with your name:
```bash
vos> mkdir /home/yourname
```
(Replace "yourname" with your actual name!)
</details>

<details>
<summary>✅ Full Solution</summary>

```bash
vos> mkdir /home
vos> mkdir /home/yourname
vos> mkdir /home/yourname/work
vos> mkdir /home/yourname/personal
vos> touch /home/yourname/work/project.txt
vos> touch /home/yourname/personal/notes.txt
vos> touch /home/yourname/readme.txt
vos> ls /home/yourname
vos> ls /home/yourname/work
vos> ls /home/yourname/personal
```
</details>

**Check your work:**
```bash
vos> cd /home/yourname
vos> ls
vos> ls work
vos> ls personal
```

Did you see all your files? 🎉

---

## 🎓 What You Just Learned!

Congrats! You just learned:

✅ What VOS is (a computer inside your computer)
✅ How to start VOS
✅ Basic commands: `echo`, `help`, `pwd`, `cd`, `ls`, `mkdir`, `touch`
✅ How directories work (folders in folders!)
✅ How to navigate the filesystem
✅ How to create files and directories

### The Mental Model

You now understand that a computer filesystem is like a tree:

```
Root /
├── Branch (directory)
│   ├── Leaf (file)
│   └── Smaller branch (subdirectory)
│       └── Leaf (file)
└── Another branch
    └── Leaf
```

You can:
- Move around the tree (`cd`)
- See where you are (`pwd`)
- Look around (`ls`)
- Add branches (`mkdir`)
- Add leaves (`touch`)

---

## 🚀 Quick Reference Card

Copy this for later:

```
┌──────────────────────────────────────────────┐
│         VOS COMMANDS YOU KNOW NOW            │
├──────────────────────────────────────────────┤
│ echo <text>      Say something              │
│ help             Show all commands           │
│ pwd              Where am I?                 │
│ ls [path]        What's here?                │
│ cd <path>        Go somewhere                │
│ cd ..            Go up one level             │
│ mkdir <path>     Make a folder               │
│ touch <file>     Create a file               │
│ exit             Leave VOS                   │
└──────────────────────────────────────────────┘
```

---

## 🎮 Try This at Home!

Before moving to Chapter 2, practice these challenges:

### Challenge 1: Your Project Structure
Create a realistic project structure:
```
/projects/
├── website/
│   ├── index.html
│   └── style.css
├── game/
│   ├── main.py
│   └── assets/
└── notes/
    └── ideas.txt
```

### Challenge 2: Navigation Practice
1. Create the structure above
2. Navigate to `/projects/game/assets`
3. Use `pwd` to verify
4. Navigate to `/projects/website` using `cd ..` commands
5. List contents

### Challenge 3: Explore!
Try combining commands:
```bash
vos> cd /projects
vos> ls
vos> cd website
vos> pwd
vos> ls
```

---

## 🤔 Common Questions

**Q: What if I make a mistake?**
A: No problem! You can:
- Use `rm <path>` to remove things
- Exit (`exit`) and start fresh
- Nothing you do in VOS affects your real computer!

**Q: Why doesn't anything appear when I run `mkdir`?**
A: In Unix/Linux style, no output = success! If there's a problem, you'll see an error message.

**Q: What's an "inode"?**
A: It's like an ID number for files. Don't worry about it for now - we'll learn in Chapter 3!

**Q: Can I write content to files?**
A: Not yet in VOS! We'd need to add that feature. For now, we can create empty files.

---

## ✨ Next Steps

You're doing great! You've mastered the basics.

**Ready for more?** → [Chapter 2: Exploring the Computer](02-exploring-the-computer.md)

In Chapter 2, you'll learn:
- What happens INSIDE the computer when you run commands
- How the CPU, memory, and files work together
- See actual diagrams of the process!

**Want to dive deeper technically?** → [Technical Tutorial Chapter 1](../../tutorials-technical/chapter-01-introduction.md)

---

## 💭 Remember

> "A computer is like a very fast, very accurate, very stupid person who follows instructions perfectly."

You're learning to give those instructions! Every time you type a command, you're telling the computer exactly what to do.

Keep practicing, stay curious, and have fun! 🚀

---

**Questions or stuck?** Open an issue on [GitHub](https://github.com/theterminalguy/vos/issues) - we're here to help!

*Made with ❤️ for curious minds*
