// NOTE: this is a default transport client which will handled by git thus agnostic of what exactly it is.
// Currently supported options by git are: https and ssl. Both of the are handled by corresponding git library.
//
// TODO: simple wrapper for client and bare server to be able to access basic push/pull operations via this code base.
//
// add  to gitignore /${dir}.git
// git  clone --bare ${dir} ${dir}.git
//
// NOTE: daemon based access:
//
// Daemon based (same as in gittorrent) read-only access seems to be the most suitable as it simplifies security setup.
// In it only the owner has write access to the local copy of their repository. In such a setup every user decides
// which version of repo to pull from external peer.
//
// The replication is achieved by users' consensus on what code is latest and most desirable.
// The issue of discoverability for the most recent code base must be resolved by users communication through
// higher protocol level.
//
// The downside might be that the most recent changes are the least replicated and their distribution through the
// network is restricted by 'spotlight'/'popularity' which might be more social rather then technical problem. 
//
// Another potential issue is that such a model might create multiple heads, the convergence of which must be done 
// manually. This will potentially lead to increase of cases for local conflict resolutions and globally different
// order of commits in the head.
//
// NOTE: for ssh based access
// Make a symlink to jailed directory (git clone works with path)
// Create a restricted user with access only to jailed directory
// Add counterparty public key to the restricted git user
