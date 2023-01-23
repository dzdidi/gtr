// NOTE: this is a default transport client which will handled by git thus agnostic of what
// exactly it is. Currently supported options by git are: https and ssl. Both of the are
// handled by corresponding git library.
// TODO: simple wrapper for client and bare server to be able to access basic push/pull operations
// via this code base
//
// add  to gitignore /${dir}.git
// git  clone --bare ${dir} ${dir}.git
// NOTE: daemon based access
// Daemon based (same as in gittorrent) read-only access seems to be the most suitable as it
// simplifies security setup. In it only owner has write access to the local copy of their
// repository. In such setup every user decides which version of repo to pull from external peer.
// In this setup the replication is achieved by users' consensus on what code is latest and most
// desirable. Issue of discoverability of the most recent code base will be resolved by users
// communication enabled by higher protocol level.
// The downside might be that the most recent changes are the least replicated and their
// distribution through the network is restricted by 'spotlight'/'popularity' which might be more
// social rather then technical problem. 
// Another potential issue is that such model might create multiple heads convergence of which must
// be done manually which will potentially lead to increase case of local conflict resolutions and
// globally different order of commits in the head.
// NOTE: for ssh based access
// make a symlink to jailed directory (git clone works with path)
// create a restricted user with access only to jailed directory
// add counterparty public key to the restricted git user
