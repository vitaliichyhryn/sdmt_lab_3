from fastapi import APIRouter
import numpy as np

router = APIRouter()

@router.get('')
def hello_world() -> dict:
    return {'msg': 'Hello, World!'}

@router.get('/matrices')
def matrix() -> dict:
    matrix_a = np.random.randint(low=0, high=128, size=(10 * 10))
    matrix_b = np.random.randint(low=0, high=128, size=(10 * 10))
    product = np.dot(matrix_a, matrix_b)
    return {"matrix_a": matrix_a, "matrix_b": matrix_b, "product": product}
