// NOTE: this is a default transport client which will handled by git thus agnostic of what
// exactly it is. Currently supported options by git are: https and ssl. Both of the are
// handled by corresponding git library.
// TODO: simple wrapper for client and bare server to be able to access basic push/pull operations
// via this code base
//
// add  to gitignore /${dir}.git
// git  clone --bare ${dir} ${dir}.git
// make a symlink to jailed directory (git clone works with path)
// create a restricted user with access only to jailed directory
// add counterparty public key to the restricted git user
