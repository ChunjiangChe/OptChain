import math
# Input hex string (with leading zeros preserved)

base_hex = "0854854854854854854854854854854854854854854854854854854854854854"
# Convert to integer
base_num = int(base_hex, 16)

multi_num = base_num // 4

# manifoldchain_ex_num = avai_num * avai_size


# Format back to hex, padded to the same length
multi_hex = f"{multi_num:0{len(base_hex)}x}"


print("Base diff :", base_hex)
print("Multi diff :", multi_hex)

# tx_num_2 = tx_num * 2
# tx_num_2_hex = f"{tx_num_2:0{len(tx_hex)}x}"
# print("Tx diff 2x :", tx_num_2_hex)

