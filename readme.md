# Vult credential sync server

## Synchronization algorithm

We assume that local changes are always the most recent, therefore overwriting any remote state.

1. Receive local app's current state id and array of mutations

2. Check the state id:

	- If state id is most recent, apply and add new mutations to databases and return new latest state id.

	- If it is missing, apply and add new mutations and return entire store

	- If found but not most recent:

		- Apply new mutations

		- Get list of remote mutations

		- Add new local mutations

		- Create hashmap of local mutations to ids of cred affected by changes or deletions

		- Filter list of remote mutations, removing anything that affects the same id

		- Return resulting list of remote mutations that are not overriden

3. While applying mutations, if any creation mutation has a duplicated id, create a new non-conflicting id and record the change in a list and return that list at the end