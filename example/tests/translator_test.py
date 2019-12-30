import origen
import origen.translator

def test_translator_can_be_initialized():
    origen.app.instantiate_dut("dut.falcon")
    remote_file = f"{origen.root}/vendor/ip-xact/spirit1-4_ip-xact.xml"
    origen.app.translate(remote_file)
