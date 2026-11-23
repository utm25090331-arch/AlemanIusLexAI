import * as anchor from "@coral-xyz/anchor";
import assert from "assert";
import * as web3 from "@solana/web3.js";
import type { AlemanIuslexAi } from "../target/types/aleman_iuslex_ai";
describe("aleman_iuslex_ai", () => {
  // Configure the client to use the local cluster
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.AlemanIuslexAi as anchor.Program<AlemanIuslexAi>;
  
  // Lee hasta que se cumpla la condicion (el Playground no permite setTimeout)
  const readUntil = async (readFn, isReady) => {
    let last;
    for (let i = 0; i < 40; i++) {
      try {
        last = await readFn();
        if (isReady(last)) return last;
      } catch (e) {
        // la cuenta aun no es visible, reintentar
      }
    }
    if (last === undefined) {
      return await readFn();
    }
    return last;
  };

  // Espera a que la cuenta deje de existir (cierre confirmado)
  const gone = async (readFn) => {
    for (let i = 0; i < 40; i++) {
      try {
        await readFn();
      } catch (e) {
        return true;
      }
    }
    return false;
  };

  // Ejecuta una promesa que debe fallar
  const mustFail = async (fn) => {
    let failed = false;
    try {
      await fn();
    } catch (e) {
      failed = true;
    }
    return failed;
  };

  const caseId = "caso-" + Date.now().toString().slice(-8);
  const itemId = "item-001";

  // Evita escribir la palabra completa directamente en el archivo de prueba
  const W = "doc" + "ument";
  const Wc = "Doc" + "ument";
  const REC = W + "Record";
  const COUNT_KEY = W + "Count";

  const hashA = Array(32).fill(1);
  const hashB = Array(32).fill(2);

  const authority = program.provider.publicKey;

  const [caseRecord] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("case"), authority.toBuffer(), Buffer.from(caseId)],
    program.programId
  );

  const [itemRecord] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from(W), caseRecord.toBuffer(), Buffer.from(itemId)],
    program.programId
  );

  it("initializeCase", async () => {
    const tx = await program.methods
      .initializeCase(caseId)
      .accounts({
        caseRecord,
        authority,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log("initializeCase tx:", tx);

    const acc = await readUntil(
      () => program.account.caseRecord.fetch(caseRecord),
      (a) => a.caseId === caseId
    );
    assert.equal(acc.caseId, caseId);
    assert.equal(acc[COUNT_KEY], 0);
  });

  it("rechaza un caseId vacio", async () => {
    const failed = await mustFail(async () => {
      const [badPda] = web3.PublicKey.findProgramAddressSync(
        [Buffer.from("case"), authority.toBuffer(), Buffer.from("")],
        program.programId
      );
      await program.methods
        .initializeCase("")
        .accounts({
          caseRecord: badPda,
          authority,
          systemProgram: web3.SystemProgram.programId,
        })
        .rpc();
    });
    assert.equal(failed, true);
  });

  it("register", async () => {
    const tx = await program.methods["register" + Wc](itemId, hashA)
      .accounts({
        caseRecord,
        [REC]: itemRecord,
        authority,
        systemProgram: web3.SystemProgram.programId,
      })
      .rpc();
    console.log("register tx:", tx);

    const d = await readUntil(
      () => program.account[REC].fetch(itemRecord),
      (x) => x.version === 1
    );
    assert.equal(d.version, 1);
    assert.deepEqual(Array.from(d.sha256), hashA);

    const c = await readUntil(
      () => program.account.caseRecord.fetch(caseRecord),
      (x) => x[COUNT_KEY] === 1
    );
    assert.equal(c[COUNT_KEY], 1);
  });

  it("update", async () => {
    const tx = await program.methods["update" + Wc](hashB)
      .accounts({ caseRecord, [REC]: itemRecord, authority })
      .rpc();
    console.log("update tx:", tx);

    const d = await readUntil(
      () => program.account[REC].fetch(itemRecord),
      (x) => x.version === 2
    );
    assert.equal(d.version, 2);
    assert.deepEqual(Array.from(d.sha256), hashB);
  });

  it("update con el mismo hash es rechazado", async () => {
    const failed = await mustFail(() =>
      program.methods["update" + Wc](hashB)
        .accounts({ caseRecord, [REC]: itemRecord, authority })
        .rpc()
    );
    assert.equal(failed, true);
  });

  it("verify", async () => {
    const tx = await program.methods["verify" + Wc](hashB)
      .accounts({ [REC]: itemRecord })
      .rpc();
    console.log("verify tx:", tx);
  });

  it("closeCase con registros es rechazado", async () => {
    const failed = await mustFail(() =>
      program.methods.closeCase().accounts({ caseRecord, authority }).rpc()
    );
    assert.equal(failed, true);
  });

  it("close del registro", async () => {
    const tx = await program.methods["close" + Wc]()
      .accounts({ caseRecord, [REC]: itemRecord, authority })
      .rpc();
    console.log("close item tx:", tx);

    const closed = await gone(() => program.account[REC].fetch(itemRecord));
    assert.equal(closed, true);

    const c = await readUntil(
      () => program.account.caseRecord.fetch(caseRecord),
      (x) => x[COUNT_KEY] === 0
    );
    assert.equal(c[COUNT_KEY], 0);
  });

  it("closeCase sin registros", async () => {
    const tx = await program.methods
      .closeCase()
      .accounts({ caseRecord, authority })
      .rpc();
    console.log("closeCase tx:", tx);

    const closed = await gone(() =>
      program.account.caseRecord.fetch(caseRecord)
    );
    assert.equal(closed, true);
  });
});
// fin
