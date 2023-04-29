Currently supported options by git are: https and ssl. Both of the are handled by corresponding git library.

TODO: simple wrapper for client and bare server to be able to access basic push/pull operations via this code base.

NOTE: daemon based access:

Daemon based (same as in gittorrent) read-only access seems to be the most suitable to start with as it simplifies security setup.
In such a setup only the owner has write access to the local copy of their repository and every user decides which version of repo to pull from external peer.

The replication is achieved by users' consensus on what code is latest and most desirable.
The issue of discoverability for the most recent code base must be resolved by users communication through higher protocol level.

The downside might be that the most recent changes are the least replicated and their distribution through the network is restricted by 'spotlight'/'popularity' which might be more social rather then technical problem.

Another potential issue is that such a model might create multiple "heads", the convergence of which must be done manually.
This will potentially lead to increase of cases for local conflict resolutions which may result in globally different order of commits in the head.

# The SSH Protocol [via](https://git-scm.com/book/en/v2/Git-on-the-Server-The-Protocols)

A common transport protocol for Git when self-hosting is over SSH. This is because SSH access to servers is already set up in most places — and if it isn’t, it’s easy to do. SSH is also an authenticated network protocol and, because it’s ubiquitous, it’s generally easy to set up and use.

To clone a Git repository over SSH, you can specify an `ssh://` URL like this:

- `$ git clone ssh://[user@]server/project.git`
- `$ git clone [user@]server:project.git`

In both cases above, if you don’t specify the optional username, Git assumes the user you’re currently logged in as.

### The Pros
The pros of using SSH are many. First, SSH is relatively easy to set up — SSH daemons are commonplace, many network admins have experience with them, and many OS distributions are set up with them or have tools to manage them. Next, access over SSH is secure — all data transfer is encrypted and authenticated. Last, like the HTTPS, Git and Local protocols, SSH is efficient, making the data as compact as possible before transferring it.

### The Cons
The negative aspect of SSH is that it doesn’t support anonymous access to your Git repository. If you’re using SSH, people must have SSH access to your machine, even in a read-only capacity, which doesn’t make SSH conducive to open source projects for which people might simply want to clone your repository to examine it. If you’re using it only within your corporate network, SSH may be the only protocol you need to deal with. If you want to allow anonymous read-only access to your projects and also want to use SSH, you’ll have to set up SSH for you to push over but something else for others to fetch from.

## SSH server setup [via](https://git-scm.com/book/en/v2/Git-on-the-Server-Getting-Git-on-a-Server)

To collaborate with a couple of people on a private project, all you need is an SSH server and a bare repository. Note that git push access for user requires SSH write access
Git will automatically add group write permissions to a repository properly if you run the `git init` command with the `--shared` option.

One method is to create a single 'git' user account on the machine, ask every user who is to have write access to send you an SSH public key, and add that key to the `~/.ssh/authorized_keys` file of that new 'git' account.
At that point, everyone will be able to access that machine via the 'git' account. This doesn’t affect the commit data in any way — the SSH user you connect as doesn’t affect the commits you’ve recorded.

More on server setup [here](https://git-scm.com/book/en/v2/Git-on-the-Server-Setting-Up-the-Server)

You should note that currently all these users can also log into the server and get a shell as the git user. If you want to restrict that, you will have to change the shell to something else in the `/etc/passwd` file. You can easily restrict the git user account to only Git-related activities with a limited shell tool called `git-shell` that comes with Git. If you set this as the git user account’s login shell, then that account can’t have normal shell access to your server. To use this, specify `git-shell` instead of `bash` or `csh` for that account’s login shell. To do so, you must first add the full pathname of the `git-shell` command to `/etc/shells` if it’s not already there:

```
$ cat /etc/shells   # see if git-shell is already in there. If not...
$ which git-shell   # make sure git-shell is installed on your system.
$ sudo -e /etc/shells  # and add the path to git-shell from last command
```

Now you can edit the shell for a user using `chsh <username> -s <shell>`:

```
$ sudo chsh git -s $(which git-shell)
```

At this point, users are still able to use SSH port forwarding to access any host the git server is able to reach. If you want to prevent that, you can edit the `~/.ssh/authorized_keys` file and prepend the following options to each key you’d like to restrict:

```
no-port-forwarding,no-X11-forwarding,no-agent-forwarding,no-pty
```

 Run `git help shell` for more information on customizing the shell.

If you’ve allowed everyone to connect with a single user (like “git”) via public-key authentication, you may have to give that user a shell wrapper that determines which user is connecting based on the public key, and set an environment variable accordingly.

