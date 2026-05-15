import subprocess


def run_ollama(model, prompt):
    result = subprocess.run(
        ["ollama", "run", model],
        input=prompt.encode(),
        stdout=subprocess.PIPE,
    )
    return result.stdout.decode()


def pqc_wrap(text):
    return f"[PQC-WRAPPED]\\n{text}\\n[/PQC-WRAPPED]"


def handle_request(prompt):
    response = run_ollama("hermes", prompt)
    return pqc_wrap(response)


if __name__ == "__main__":
    print(handle_request("Hello from PQC Hermes Agent"))

