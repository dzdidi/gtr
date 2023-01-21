# The SSH Protocol
 via https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols
A common transport protocol for Git when self-hosting is over SSH. This is because SSH access to servers is already set up in most places — and if it isn’t, it’s easy to do. SSH is also an authenticated network protocol and, because it’s ubiquitous, it’s generally easy to set up and use.

To clone a Git repository over SSH, you can specify an ssh:// URL like this:

`$ git clone ssh://[user@]server/project.git`
Or you can use the shorter scp-like syntax for the SSH protocol:

`$ git clone [user@]server:project.git`
In both cases above, if you don’t specify the optional username, Git assumes the user you’re currently logged in as.

### The Pros
The pros of using SSH are many. First, SSH is relatively easy to set up — SSH daemons are commonplace, many network admins have experience with them, and many OS distributions are set up with them or have tools to manage them. Next, access over SSH is secure — all data transfer is encrypted and authenticated. Last, like the HTTPS, Git and Local protocols, SSH is efficient, making the data as compact as possible before transferring it.

### The Cons
The negative aspect of SSH is that it doesn’t support anonymous access to your Git repository. If you’re using SSH, people must have SSH access to your machine, even in a read-only capacity, which doesn’t make SSH conducive to open source projects for which people might simply want to clone your repository to examine it. If you’re using it only within your corporate network, SSH may be the only protocol you need to deal with. If you want to allow anonymous read-only access to your projects and also want to use SSH, you’ll have to set up SSH for you to push over but something else for others to fetch from.

## SSH server setup: via https://git-scm.com/book/en/v2/Git-on-the-Server-Getting-Git-on-a-Server
To collaborate with a couple of people on a private project, all you need is an SSH server and a bare repository.
git push access for user requires SSH write access
Git will automatically add group write permissions to a repository properly if you run the git init command with the --shared option.
TODO: check what group exactly will be created

One method is to create a single 'git' user account on the machine, ask every user who is to have write access to send you an SSH public key, and add that key to the ~/.ssh/authorized_keys file of that new 'git' account. At that point, everyone will be able to access that machine via the 'git' account. This doesn’t affect the commit data in any way — the SSH user you connect as doesn’t affect the commits you’ve recorded.

NOTE: more on server setup via https://git-scm.com/book/en/v2/Git-on-the-Server-Setting-Up-the-Server
You should note that currently all these users can also log into the server and get a shell as the git user. If you want to restrict that, you will have to change the shell to something else in the /etc/passwd file.

You can easily restrict the git user account to only Git-related activities with a limited shell tool called git-shell that comes with Git. If you set this as the git user account’s login shell, then that account can’t have normal shell access to your server. To use this, specify git-shell instead of bash or csh for that account’s login shell. To do so, you must first add the full pathname of the git-shell command to /etc/shells if it’s not already there:

```
$ cat /etc/shells   # see if git-shell is already in there. If not...
$ which git-shell   # make sure git-shell is installed on your system.
$ sudo -e /etc/shells  # and add the path to git-shell from last command
```
Now you can edit the shell for a user using chsh <username> -s <shell>:
```
$ sudo chsh git -s $(which git-shell)
```

At this point, users are still able to use SSH port forwarding to access any host the git server is able to reach. If you want to prevent that, you can edit the authorized_keys file and prepend the following options to each key you’d like to restrict:

```
no-port-forwarding,no-X11-forwarding,no-agent-forwarding,no-pty
```
 Run `git help shell` for more information on customizing the shell.

If you’ve allowed everyone to connect with a single user (like “git”) via public-key authentication, you may have to give that user a shell wrapper that determines which user is connecting based on the public key, and set an environment variable accordingly.

# Git Daemon
The Git Protocol (this is what gittorrent implementation is based on):
Finally, we have the Git protocol. This is a special daemon that comes packaged with Git; it listens on a dedicated port (9418) that provides a service similar to the SSH protocol, but with absolutely no authentication. In order for a repository to be served over the Git protocol, you must create a git-daemon-export-ok file — the daemon won’t serve a repository without that file in it — but, other than that, there is no security. Either the Git repository is available for everyone to clone, or it isn’t. This means that there is generally no pushing over this protocol. You can enable push access but, given the lack of authentication, anyone on the internet who finds your project’s URL could push to that project. Suffice it to say that this is rare.

### The Pros
The Git protocol is often the fastest network transfer protocol available. If you’re serving a lot of traffic for a public project or serving a very large project that doesn’t require user authentication for read access, it’s likely that you’ll want to set up a Git daemon to serve your project. It uses the same data-transfer mechanism as the SSH protocol but without the encryption and authentication overhead.

### The Cons
The downside of the Git protocol is the lack of authentication. It’s generally undesirable for the Git protocol to be the only access to your project. Generally, you’ll pair it with SSH or HTTPS access for the few developers who have push (write) access and have everyone else use git:// for read-only access. It’s also probably the most difficult protocol to set up. It must run its own daemon, which requires xinetd or systemd configuration or the like, which isn’t always a walk in the park. It also requires firewall access to port 9418, which isn’t a standard port that corporate firewalls always allow. Behind big corporate firewalls, this obscure port is commonly blocked.

## Deamon setup via https://git-scm.com/book/en/v2/Git-on-the-Server-Git-Daemon
Next we’ll set up a daemon serving repositories using the “Git” protocol. This is a common choice for fast, unauthenticated access to your Git data. Remember that since this is not an authenticated service, anything you serve over this protocol is public within its network.

If you’re running this on a server outside your firewall, it should be used only for projects that are publicly visible to the world. If the server you’re running it on is inside your firewall, you might use it for projects that a large number of people or computers (continuous integration or build servers) have read-only access to, when you don’t want to have to add an SSH key for each.

In any case, the Git protocol is relatively easy to set up. Basically, you need to run this command in a daemonized manner:

```
$ git daemon --reuseaddr --base-path=/srv/git/ /srv/git/
```
The --reuseaddr option allows the server to restart without waiting for old connections to time out, while the --base-path option allows people to clone projects without specifying the entire path, and the path at the end tells the Git daemon where to look for repositories to export. If you’re running a firewall, you’ll also need to punch a hole in it at port 9418 on the box you’re setting this up on.

You can daemonize this process a number of ways, depending on the operating system you’re running.

Since systemd is the most common init system among modern Linux distributions, you can use it for that purpose. Simply place a file in /etc/systemd/system/git-daemon.service with these contents:

```
[Unit]
Description=Start Git Daemon

[Service]
ExecStart=/usr/bin/git daemon --reuseaddr --base-path=/srv/git/ /srv/git/

Restart=always
RestartSec=500ms

StandardOutput=syslog
StandardError=syslog
SyslogIdentifier=git-daemon

User=git
Group=git

[Install]
WantedBy=multi-user.target
```
You might have noticed that Git daemon is started here with git as both group and user. Modify it to fit your needs and make sure the provided user exists on the system. Also, check that the Git binary is indeed located at /usr/bin/git and change the path if necessary.

Finally, you’ll run `systemctl enable git-daemon to automatically start the service on boot, and can start and stop the service with, respectively, systemctl start git-daemon and systemctl stop git-daemon.

On other systems, you may want to use xinetd, a script in your sysvinit system, or something else — as long as you get that command daemonized and watched somehow.

Next, you have to tell Git which repositories to allow unauthenticated Git server-based access to. You can do this in each repository by creating a file named git-daemon-export-ok.

```
$ cd /path/to/project.git
$ touch git-daemon-export-ok
```
The presence of that file tells Git that it’s OK to serve this project without authentication.
