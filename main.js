let OPT_CACHE = {}

function send(name, args, zeroCopy){
	const id = OPT_CACHE[name]
}

function sendSync(name, args, zeroCopy){
	const id = OPT_CACHE[name]
}

function kill(signal, pid){
	return sendSync('op_kill', {pid, signal})
}

/*
 * run a script
 *
 * {object} req - request object
 * {array} req.cmd - command
 * {string} [req.cwd] - current working dir
 * {array} [req.env] - env var
 * {string} req.stdin - stdin
 * {string} req.stdout - stdout
 * {string} req.stderr - stdout
 * {number} req.stdinRid - stdin
 * {number} req.stdoutRid - stdout
 * {number} req.stderrRid - stdout
 * {object} - response object {rid, pid, stdinRid, stdoutRid, stderrRid}
 */
function run(req){
	assert(req.cmd && assert(req.cmd.length > 0)
	return sendSync('op_run', {pid, signal})
}

function runStatus(rid){
	return send('op_run_status', {rid})
}

async main(){
	OPT_CACHE = Deno.core.ops()
	for (let name in ops){
		console.log(name)
	}
}

main()
